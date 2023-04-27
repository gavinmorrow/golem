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

#[derive(Debug, serde::Deserialize)]
/// Basically just a [`Message`](crate::model::Message) without an id.
pub struct SendMessage {
	pub author: crate::model::user::Id,
    pub parent: Option<crate::model::message::Id>,
    pub content: String,
}

#[debug_handler]
pub async fn post_message(
    State(state): State<Arc<AppState>>,
    Json(message): Json<SendMessage>,
) -> Result<String, StatusCode> {
	// Generate an id
	let id = state.snowcloud.next_id().expect("Failed to generate snowflake.");
	let id_string = id.id().to_string();

	// Create a Message
	let message = Message {
		id,
		author: message.author,
		parent: message.parent,
		content: message.content,
	};
	
	// Add to database
	let database = state.database.lock().await;
	match database.add_message(message) {
		Ok(()) => Ok(id_string),
		Err(err) => {
			error!("Failed to add message to database: {:?}", err);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		}
	}
}
