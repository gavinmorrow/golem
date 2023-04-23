use crate::{
    auth,
    model::{AppState, User},
};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use log::{debug, error, warn};
use std::sync::Arc;

pub mod sessions;

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

#[derive(Debug, serde::Deserialize)]
struct PartialUser {
    pub name: String,
    pub password: String,
}

#[debug_handler]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(user): Json<PartialUser>,
) -> Result<String, StatusCode> {
    let snowflake = state.snowcloud.next_id();
    if let Err(err) = snowflake {
        return match err {
            snowcloud::Error::SequenceMaxReached(_next_millisecond) => {
                warn!("Snowflake sequence max reached: {}", err);
                Err(StatusCode::TOO_MANY_REQUESTS)
            }
            _ => {
                error!("Failed to generate snowflake: {}", err);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        };
    }

    let snowflake = snowflake.unwrap();
    let password = auth::hash::hash_password(user.password);
    let user = User {
        id: snowflake.clone(),
        name: user.name,
        password,
    };

    let database = state.database.lock().await;

    if let Err(err) = database.add_user(user) {
        error!("Failed to add user to database: {:?}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(snowflake.id().to_string())
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
