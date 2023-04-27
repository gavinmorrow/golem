use crate::model::{AppState, User};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use log::{debug, error};
use std::sync::Arc;

pub mod auth;
pub mod messages;
pub mod register;
pub mod sessions;
pub mod ws;

#[debug_handler]
pub async fn snowflake(State(state): State<Arc<AppState>>) -> String {
    let snowflake = state.snowcloud.next_id();
    match snowflake {
        Ok(snowflake) => {
            let id = snowflake.id();
            let (ts, pid, seq) = snowflake.into_parts();
            format!(
                "Snowflake: {:#043b} {:#08b} {:#012b} ({})",
                ts, pid, seq, id
            )
        }
        Err(err) => format!("Failed to generate snowflake: {}", err),
    }
}

#[debug_handler]
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<super::model::user::Id>,
) -> Result<Json<User>, StatusCode> {
    let database = state.database.lock().await;

    let user = match database.get_user(&id) {
        Ok(Some(user)) => user,
        Ok(None) => {
            debug!("User {:?} not found in database.", id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(err) => {
            error!("Failed to get user from database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(user))
}
