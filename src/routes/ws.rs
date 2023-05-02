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
use tokio::sync::broadcast;

use crate::{
    auth,
    model::{AppState, Message, Session},
};

struct WsState {
    appstate: Arc<AppState>,
    tx: broadcast::Sender<ServerMsg>,
}

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let (tx, _) = broadcast::channel(10);
    let state = Arc::new(WsState {
        appstate: state,
        tx,
    });

    Router::<Arc<WsState>>::new()
        .route("/", get(handler))
        .with_state(state)
}

#[debug_handler]
async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<WsState>>) -> Response {
    trace!("ws connection requested");
    ws.on_upgrade(move |ws| handle_ws(ws, state.appstate.clone(), state.tx.clone()))
}

// Naming note (for types and variables):
//
// - Use `msg` as an abbreviation for `message`
//   when you're referring to a websocket message
//
// - Use `message` when you're referring to a
//   message in the database/chat

async fn handle_ws(mut ws: WebSocket, state: Arc<AppState>, tx: broadcast::Sender<ServerMsg>) {
    trace!("ws connection opened");

    let mut session: Option<Session> = None;
    let mut rx = tx.subscribe();

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
                debug!("sending message: {:?}", msg);
                if ws.send(Into::<String>::into(msg).into()).await.is_err() {
                    // client disconnected
                    break;
                }
            }
            HandlerResult::Broadcast(msg) => {
                debug!("broadcasting message: {:?}", msg);
                if tx.send(msg).is_err() {
                    // client disconnected
                    break;
                }
            }
        }
    }

    while let Ok(msg) = rx.recv().await {
        debug!("sending message from broadcast: {:?}", msg);
        if ws.send(Into::<String>::into(msg).into()).await.is_err() {
            // client disconnected
            break;
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
        ClientMsg::AuthenticateToken(token) => {
            let database = state.database.lock().await;
            let Ok(session_db) = auth::verify_session(token, database) else {
				// Client sent invalid token
				return Reply(ServerMsg::Authenticate { success: false });
			};

            *session = Some(session_db);

            // Client sent valid token
            // Respond with success
            Reply(ServerMsg::Authenticate { success: true })
        }
        ClientMsg::Authenticate(user) => {
            let database = state.database.lock().await;

            // Get correct password hash
            let user_db = match database.get_user_by_name(&user.name) {
                Ok(Some(user)) => user,
                Ok(None) => {
                    // User doesn't exist
                    return Reply(ServerMsg::Authenticate { success: false });
                }
                Err(err) => {
                    error!("Failed to get user from database: {}", err);
                    return Reply(ServerMsg::Error);
                }
            };

            // Check credentials
            let success = auth::hash::check_passwords(user.password, user_db.password);

            // set session
            *session = Some(Session::generate(state.next_snowflake(), user_db.id));

            Reply(ServerMsg::Authenticate { success })
        }
        ClientMsg::Pong => Continue,
        ClientMsg::Message(message) => {
            // Generate an id
            let id = state.next_snowflake();

            // Get author from session
            if session.is_none() {
                return Reply(ServerMsg::Error);
            }
            let author = session.clone().unwrap().user_id;

            // Create a Message
            let message = Message {
                id,
                author,
                parent: message.parent,
                content: message.content,
            };

            // Add to database
            let database = state.database.lock().await;
            match database.add_message(&message) {
                Ok(()) => Broadcast(ServerMsg::NewMessage(message)),
                Err(err) => {
                    error!("Failed to add message to database: {:?}", err);
                    return Reply(ServerMsg::Error);
                }
            }
        }
        ClientMsg::LoadMessages { before, amount } => {
            let database = state.database.lock().await;
            match database.get_messages(before, amount) {
                Ok(messages) => Reply(ServerMsg::Messages(messages)),
                Err(err) => {
                    error!("Failed to get messages from database: {:?}", err);
                    Reply(ServerMsg::Error)
                }
            }
        }
        ClientMsg::LoadChildren { parent, depth } => {
            let database = state.database.lock().await;
            match database.get_children_of(Some(&parent), depth) {
                Ok(messages) => Reply(ServerMsg::Messages(messages)),
                Err(err) => {
                    error!("Failed to get messages from database: {:?}", err);
                    Reply(ServerMsg::Error)
                }
            }
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
/// Basically just a [`Message`](crate::model::Message) without an id.
pub struct SendMessage {
    // pub author: crate::model::user::Id,
    pub parent: Option<crate::model::message::Id>,
    pub content: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct PartialUser {
    pub name: String,
    pub password: String,
}

enum HandlerResult {
    Continue,
    Reply(ServerMsg),
    Broadcast(ServerMsg),
}

#[derive(Clone, Debug, serde::Deserialize)]
enum ClientMsg {
    AuthenticateToken(u64),
    Authenticate(PartialUser),
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

#[derive(Clone, Debug, serde::Serialize)]
enum ServerMsg {
    Authenticate { success: bool },
    NewMessage(Message),
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
