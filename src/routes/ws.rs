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
use log::{debug, trace};

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

    let mut session = None;

    while let Some(msg) = ws.recv().await {
        let Ok(msg) = msg else {
            // client disconnected
            break;
        };

        let Ok(msg) = ClientMsg::build(msg) else {
			// client sent invalid message, ignore
			debug!("client sent invalid message: {:?}", msg);
			continue;
		};

        debug!("received message: {:?}", msg);
    }

    trace!("ws connection closed");
}

async fn handle_message(
    msg: ClientMsg,
    session: Option<Session>,
    state: Arc<AppState>,
) -> HandlerResult {
	use HandlerResult::*;

    match msg {
        ClientMsg::Authenticate { token } => {
            let database = state.database.lock().await;
            let Ok(session) = auth::verify_session(token, database) else {
					// Client sent invalid token
					return Reply(ServerMsg::Authenticate { success: false });
				};

            // Client sent valid token
            // Respond with success
            return Reply(ServerMsg::Authenticate { success: true });
        }
        ClientMsg::Pong => Continue,
        ClientMsg::Message { message } => {}
    }
}

enum HandlerResult {
    Continue,
    Break,
    Reply(ServerMsg),
}

#[derive(Debug, serde::Deserialize)]
enum ClientMsg {
    Authenticate { token: u64 },
    Pong,
    Message { message: Message },
}

#[derive(Debug, serde::Serialize)]
enum ServerMsg {
    Authenticate { success: bool },
    Ping,
    Message { message: Message },
}

impl Into<String> for ServerMsg {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl ClientMsg {
    fn build(Text(msg): ws::Message) -> Result<ClientMsg, serde_json::Error> {
        serde_json::from_str(&msg)
    }
}
