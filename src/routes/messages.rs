use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use axum_macros::debug_handler;
use log::error;

use crate::model::{AppState, Message};

#[debug_handler]
pub async fn get_snapshot(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    // Fetch the last 100 messages from the database
    let database = state.database.lock().await;
    match database.get_recent_messages() {
        Ok(messages) => Ok(Json(messages)),
        Err(err) => {
            error!("Failed to get messages from database: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
