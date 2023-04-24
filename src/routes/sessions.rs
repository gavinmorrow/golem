use crate::{
    auth,
    model::{AppState, Session},
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use log::{debug, error, trace};
use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
pub struct CreateUser {
    name: String,
    password: String,
}

#[debug_handler]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(user): Json<CreateUser>,
) -> Result<String, StatusCode> {
    // Get id
    let db = state.database.lock().await;
    let user_db = match db.get_user_by_name(&user.name) {
        Ok(Some(user)) => user,
        Ok(None) => {
            debug!("User not found: {}", user.name);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(err) => {
            error!("Failed to get user from database: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check password
    if !auth::hash::check_passwords(user.password, user_db.password) {
        trace!("Password incorrect for user: {}", user_db.name);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate token
    let id = state.snowcloud.next_id().unwrap();
    let token = auth::token::generate_token();
    let session = Session::new(id, token.clone(), user_db.id);

    // Add token to database
    if let Err(err) = db.add_session(session) {
        error!("Failed to add token to database: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(token.to_string())
}
