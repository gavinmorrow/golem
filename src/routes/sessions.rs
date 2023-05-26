use crate::{
    auth,
    model::{AppState, Session},
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use axum_macros::debug_handler;
use log::{debug, error};
use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
pub struct CreateUser {
    name: String,
    password: String,
}

#[debug_handler]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(user_body): Json<CreateUser>,
) -> Response {
    debug!("Got login request for user: {}", user_body.name);

    // Get id
    let db = state.database.lock().await;
    let user_db = match db.get_user_by_name(&user_body.name) {
        Ok(Some(user)) => user,
        Ok(None) => {
            debug!("User not found: {}", user_body.name);
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(err) => {
            error!("Failed to get user from database: {}", err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Check password
    if !auth::hash::check_passwords(user_body.password, user_db.password) {
        debug!("Password incorrect for user: {}", user_db.name);
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Generate token
    let id = state.next_snowflake();
    let token = auth::token::generate_token();
    let session = Session::new(id, token, user_db.id);

    debug!("Logging in user with session {}", session.id.id());

    // Add token to database
    if let Err(err) = db.add_session(session) {
        error!("Failed to add token to database: {}", err);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let cookie = make_cookie(token);
    let response = make_response(cookie);
    response
}

fn make_cookie(token: crate::model::session::Token) -> String {
    format!(
        // In production, the secure flag should be present
        "token={}; HttpOnly; SameSite=Lax; Path=/;",
        token,
    )
}

fn make_response(cookie: String) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        cookie.parse().expect("cookie was hard coded"),
    );

    response
}

#[debug_handler]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<Session>,
) -> StatusCode {
    debug!("Logging out session: {:?}", session.id.id());

    let database = state.database.lock().await;
    let id = session.id;

    match database.delete_session(&id) {
        Ok(_) => StatusCode::RESET_CONTENT,
        Err(err) => {
            error!("Failed to delete session from database: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
