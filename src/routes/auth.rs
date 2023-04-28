use axum::{
    extract::{State, TypedHeader},
    headers::{Cookie},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use log::{trace};

use crate::{
    auth,
    model::{session::Token, AppState},
};

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
    let session = match auth::verify_session(token, database) {
        Ok(session) => session,
        Err(crate::auth::verify_session::Error::SessionNotFound) => {
            return StatusCode::UNAUTHORIZED.into_response()
        }
        Err(crate::auth::verify_session::Error::DatabaseError) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
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
