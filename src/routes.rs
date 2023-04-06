use crate::model::{user::PartialUser, AppState, User};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use log::{error, warn};
use std::sync::Arc;

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
    let user = User {
        id: snowflake.clone(),
        name: user.name,
    };

    let mut database = state.database.lock().await;

    if let Err(err) = database.add_user(user) {
        eprintln!("Failed to add user to database: {:?}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(snowflake.id().to_string())
}
