use std::sync::Mutex;

use axum::{
    extract::{State, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use log::{debug, error};
use tokio::sync::MutexGuard;

use crate::model::{session::Token, AppState, Database, Session};

pub async fn authenticate<B>(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    // Parse token
    let token = auth_header.token();
    let token = parse_token(token).unwrap();

    let database = state.database.lock().await;
    let session = verify_session(token, database).unwrap();
    request.extensions_mut().insert(Mutex::new(session));

    // Continue
    let response = next.run(request).await;
    response
}

fn parse_token(token: &str) -> Result<Token, StatusCode> {
    let token = token.parse::<u64>();

    if token.is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(token.unwrap())
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
