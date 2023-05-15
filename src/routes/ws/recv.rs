use std::sync::Arc;

use axum::extract::ws::{self, WebSocket};
use futures::StreamExt;
use log::{debug, trace};
use tokio::sync::broadcast;

use crate::model::{AppState, Session};

use super::{
    broadcast_msg::{self, BroadcastMsg},
    Broadcast,
};

use msg::ClientMsg;
use msg_handler::HandlerResult;

mod msg;
mod msg_handler;

pub(super) async fn recv_ws(
    mut receiver: futures::stream::SplitStream<WebSocket>,
    mut session: Option<Session>,
    state: Arc<AppState>,
    id: i64,
    tx: broadcast::Sender<Broadcast>,
) {
    let mut dedup_ids = Vec::new();

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

        let msg_responses =
            msg_handler::handle_message(msg, &mut session, &mut dedup_ids, state.clone()).await;

        match msg_responses {
            Some(msg_responses) => {
                for response in msg_responses {
                    match response {
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
            None => continue,
        }
    }
}
