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
use futures::{SinkExt, StreamExt};
use log::{debug, error, trace};
use tokio::sync::broadcast;

use crate::{
    auth,
    model::{AppState, Message, Session},
};

use self::{broadcast_msg::BroadcastMsg, state::WsState};

mod recv;
mod state;

mod broadcast_msg {
    #[derive(Clone)]
    pub struct BroadcastMsg<T> {
        pub target: Target,
        pub content: T,
    }

    #[derive(Clone)]
    pub enum Target {
        All,
        One(i64),
    }
}

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

async fn handle_ws(ws: WebSocket, state: Arc<AppState>, tx: Sender) {
    trace!("ws connection opened");

    let session: Option<Session> = None;
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

async fn broadcast_handler(
    mut rx: broadcast::Receiver<Broadcast>,
    id: i64,
    mut sender: futures::stream::SplitSink<WebSocket, ws::Message>,
) {
    while let Ok(msg) = rx.recv().await {
        // Check the target
        // If it's all or the current id, send it
        // Otherwise ignore it
        match msg.target {
            broadcast_msg::Target::All => {
                debug!("sending message from broadcast: {:?}", msg.content)
            }
            broadcast_msg::Target::One(target_id) => {
                if id == target_id {
                    // Yup! A special message just for us!
                    debug!("sending message to ws {}: {:?}", id, msg.content);
                } else {
                    // Not for us
                    continue;
                }
            }
        }
        if sender
            .send(Into::<String>::into(msg.content).into())
            .await
            .is_err()
        {
            // client disconnected
            break;
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

#[derive(Clone, Debug, serde::Deserialize)]
enum ClientMsg {
    AuthenticateToken(u64),
    Authenticate(PartialUser),
    Pong,
    Message(SendMessage),
    LoadAllMessages,
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
    fn build(msg: ws::Message) -> Result<ClientMsg, ClientMsgBuildError> {
        let Text(msg) = msg else {
            return Err(ClientMsgBuildError::MsgType);
        };

        match serde_json::from_str(&msg) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(ClientMsgBuildError::Serde(err)),
        }
    }
}

#[derive(Debug)]
enum ClientMsgBuildError {
    MsgType,
    Serde(serde_json::error::Error),
}
