use axum::{
    extract::{State, TypedHeader},
    headers::{authorization::Bearer, Authorization, Cookie},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use log::{debug, error, trace};
use tokio::sync::MutexGuard;

use crate::model::{session::Token, AppState, Database, Session};

pub async fn authenticate<B>(
    TypedHeader(cookies): TypedHeader<Cookie>,
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    // Get token out of cookies
    let Some(token) = cookies.get("token") else {
        trace!("No token cookie found");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // Parse token
    let Some(token) = parse_token(token) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let database = state.database.lock().await;
    let session = match verify_session(token, database) {
        Ok(session) => session,
        Err(status_code) => return status_code.into_response(),
    };

    request.extensions_mut().insert(session);

    trace!("Request authenticated with a session token of {}", token);

    // Continue
    let response = next.run(request).await;
    response
}

fn parse_token(token: &str) -> Option<Token> {
    token.parse::<u64>().ok()
}

fn verify_session(token: u64, database: MutexGuard<Database>) -> Result<Session, StatusCode> {
    // Get and verify session
    match database.get_session_from_token(&token) {
        Ok(Some(session)) => return Ok(session),
        Ok(None) => {
            debug!("Session {} not found in database", token);
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(err) => {
            error!("Failed to get session from database: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
}
