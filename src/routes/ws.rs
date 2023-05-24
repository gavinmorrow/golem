use std::sync::Arc;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    headers::Cookie,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router, TypedHeader,
};
use axum_macros::debug_handler;
use futures::StreamExt;
use log::trace;
use tokio::sync::broadcast;

use crate::{
    auth,
    model::{AppState, Message, Session, User},
    routes::ws::broadcast_handler::broadcast_handler,
};

use self::{broadcast_msg::BroadcastMsg, state::WsState};

mod broadcast_handler;
mod broadcast_msg;
mod recv;
mod state;

type Broadcast = BroadcastMsg<ServerMsg>;
type Sender = broadcast::Sender<Broadcast>;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(WsState {
        appstate: state,
        tx,
    });

    Router::<Arc<WsState>>::new()
        .route("/", get(handler))
        .with_state(state)
}

#[debug_handler]
async fn handler(
    TypedHeader(cookies): TypedHeader<Cookie>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsState>>,
) -> Response {
    trace!("ws connection requested");

    let database = state.appstate.database.lock().await;
    let session = match crate::routes::auth::get_session_token(cookies) {
        Some(token) => match auth::verify_session(token, database) {
            Ok(session) => {
                trace!(
                    "Request authenticated with a session token of {}",
                    session.token
                );
                Some(session)
            }
            Err(crate::auth::verify_session::Error::SessionNotFound) => {
                trace!("Session not found. Request continuing without authentication");
                None
            }
            Err(crate::auth::verify_session::Error::DatabaseError) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        None => {
            trace!("Request continuing without authentication");
            None
        }
    };

    let appstate = state.appstate.clone();
    let tx = state.tx.clone();
    ws.on_upgrade(move |ws| handle_ws(ws, appstate, session, tx))
}

// Naming note (for types and variables):
//
// - Use `msg` as an abbreviation for `message`
//   when you're referring to a websocket message
//
// - Use `message` when you're referring to a
//   message in the database/chat

async fn handle_ws(ws: WebSocket, state: Arc<AppState>, session: Option<Session>, tx: Sender) {
    trace!("ws connection opened");

    let id = state.next_snowflake().id();

    // Split ws to send and receive at the same time
    let (sender, receiver) = ws.split();
    let rx = tx.subscribe();

    // Send messages
    let mut send_task = tokio::spawn(broadcast_handler(rx, id, sender));

    let tx = tx.clone();

    let mut recv_task = tokio::spawn(recv::recv_ws(receiver, session, state, id, tx));

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    trace!("ws connection closed");
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum ServerMsg {
    Authenticate { success: bool },
    NewMessage(Message),
    Error,
    Messages(Vec<Message>),
    Duplicate(String),
    Unauthenticated,
    Join(User),
}

impl Into<String> for ServerMsg {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
