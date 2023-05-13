use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use axum_macros::debug_handler;
use log::{error, warn};

use crate::{
    auth,
    model::{AppState, Snowflake, User},
};

#[derive(Debug, serde::Deserialize)]
pub struct PartialUser {
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

    let snowflake: Snowflake = snowflake.unwrap().into();
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
