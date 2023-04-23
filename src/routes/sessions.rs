use crate::model::{AppState, User};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use axum_macros::debug_handler;
use log::{error, warn, debug};
use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
struct CreateUser {
	name: String,
	password: String,
}

#[debug_handler]
pub async fn login(
    State(state): State<Arc<AppState>>,
	Json(user): Json<CreateUser>,
) -> Result<String, StatusCode> {
	
}
