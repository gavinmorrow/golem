use std::sync::Arc;

use axum::{
    extract::{
        ws::{self, Message::Text, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use log::{debug, error, trace};

use crate::{
    auth,
    model::{AppState, Message, Session},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new().route("/", get(handler))
}

#[debug_handler]
async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    trace!("ws connection requested");
    ws.on_upgrade(|ws| handle_ws(ws, state))
}

// Naming note (for types and variables):
//
// - Use `msg` as an abbreviation for `message`
//   when you're referring to a websocket message
//
// - Use `message` when you're referring to a
//   message in the database/chat

async fn handle_ws(mut ws: WebSocket, state: Arc<AppState>) {
    trace!("ws connection opened");

    let mut session: Option<Session> = None;

    while let Some(msg) = ws.recv().await {
        let Ok(msg) = msg else {
            // client disconnected
            break;
        };

        let msg = match ClientMsg::build(msg.clone()) {
            Ok(msg) => msg,
            Err(err) => {
                // client sent invalid message, ignore
                debug!("client sent invalid message: {:?}\nError: {}", msg, err);
                continue;
            }
        };

        debug!("received message: {:?}", msg);

        match handle_message(msg, &mut session, state.clone()).await {
            HandlerResult::Continue => continue,
            HandlerResult::Reply(msg) => {
                if ws.send(Into::<String>::into(msg).into()).await.is_err() {
                    // client disconnected
                    break;
                }
            }
        }
    }

    trace!("ws connection closed");
}

async fn handle_message(
    msg: ClientMsg,
    session: &mut Option<Session>,
    state: Arc<AppState>,
) -> HandlerResult {
    use HandlerResult::*;

    match msg {
        ClientMsg::Authenticate { token } => {
            let database = state.database.lock().await;
            let Ok(session_db) = auth::verify_session(token, database) else {
				// Client sent invalid token
				return Reply(ServerMsg::Authenticate { success: false });
			};

            *session = Some(session_db);

            // Client sent valid token
            // Respond with success
            return Reply(ServerMsg::Authenticate { success: true });
        }
        ClientMsg::Pong => Continue,
        ClientMsg::Message(message) => {
            // Generate an id
            let id = state
                .snowcloud
                .next_id()
                .expect("Failed to generate snowflake.");

            // Create a Message
            let message = Message {
                id,
                author: message.author,
                parent: message.parent,
                content: message.content,
            };

            // Add to database
            let database = state.database.lock().await;
            match database.add_message(&message) {
                Ok(()) => Reply(ServerMsg::Message(message)),
                Err(err) => {
                    error!("Failed to add message to database: {:?}", err);
                    todo!("Reply with error")
                }
            }
        }
        ClientMsg::LoadMessages { before, amount } => {
            let database = state.database.lock().await;
            match database.get_messages(before, amount) {
                Ok(messages) => Reply(ServerMsg::Messages(messages)),
                Err(err) => {
                    error!("Failed to get messages from database: {:?}", err);
                    return Reply(ServerMsg::Error);
                }
            }
        }
        ClientMsg::LoadChildren { parent, depth } => {
            let database = state.database.lock().await;
            match database.get_children_of(Some(&parent), depth) {
                Ok(messages) => Reply(ServerMsg::Messages(messages)),
                Err(err) => {
                    error!("Failed to get messages from database: {:?}", err);
                    return Reply(ServerMsg::Error);
                }
            }
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

enum HandlerResult {
    Continue,
    Reply(ServerMsg),
}

#[derive(Debug, serde::Deserialize)]
enum ClientMsg {
    Authenticate {
        token: u64,
    },
    Pong,
    Message(SendMessage),
    LoadMessages {
        before: Option<crate::model::message::Id>,
        amount: u8,
    },
    LoadChildren {
        parent: crate::model::message::Id,
        depth: u8,
    },
}

#[derive(Debug, serde::Serialize)]
enum ServerMsg {
    Authenticate { success: bool },
    Message(Message),
    Error,
    Messages(Vec<Message>),
}

impl Into<String> for ServerMsg {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl ClientMsg {
    /// Build a [`ClientMsg`] from a [`ws::Message`].
    /// The message must be the [`Text`](ws::Message::Text) variant.
    ///
    /// # Panics
    ///
    /// Panics if the message is not the [`Text`](ws::Message::Text) variant.
    fn build(msg: ws::Message) -> Result<ClientMsg, serde_json::Error> {
        let Text(msg) = msg else { panic!("Invalid message type") };
        serde_json::from_str(&msg)
    }
}
