use std::sync::Arc;

use axum::extract::ws::{self, WebSocket};
use futures::StreamExt;
use log::{debug, trace};
use tokio::sync::broadcast;

use crate::model::{AppState, Session};

use super::{
    broadcast_msg::{self, BroadcastMsg},
    Broadcast, ClientMsg,
};

use msg::HandlerResult;

mod msg;

pub(super) async fn recv_ws(
    mut receiver: futures::stream::SplitStream<WebSocket>,
    mut session: Option<Session>,
    state: Arc<AppState>,
    id: i64,
    tx: broadcast::Sender<Broadcast>,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        if let ws::Message::Close(_) = msg {
            // client closing
            trace!("Client sent close frame");
            break;
        }

        if let ws::Message::Pong(_) = msg {
            trace!("Client sent pong");
        }

        let msg = match ClientMsg::build(msg.clone()) {
            Ok(msg) => msg,
            Err(err) => {
                // client sent invalid message, ignore
                debug!("client sent invalid message: {:?}\nError: {:?}", msg, err);
                continue;
            }
        };

        debug!("received message: {:?}", msg);

        match msg::handle_message(msg, &mut session, state.clone()).await {
            HandlerResult::Continue => continue,
            HandlerResult::Reply(msg) => {
                debug!("sending message to {}", id);
                let msg = BroadcastMsg {
                    target: broadcast_msg::Target::One(id),
                    content: msg,
                };
                if tx.send(msg).is_err() {
                    break;
                }
            }
            HandlerResult::Broadcast(msg) => {
                trace!("broadcasting message");
                let msg = BroadcastMsg {
                    target: broadcast_msg::Target::All,
                    content: msg,
                };
                if tx.send(msg).is_err() {
                    break;
                }
            }
        }
    }
}
