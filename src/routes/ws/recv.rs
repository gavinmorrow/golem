use std::sync::Arc;

use axum::extract::ws::{self, WebSocket};
use futures::StreamExt;
use log::{debug, trace};
use tokio::sync::broadcast;

use crate::model::AppState;

use super::{
    broadcast_msg::{self, BroadcastMsg},
    presence::Presence,
    Broadcast,
};

use msg::ClientMsg;
use msg_handler::HandlerResult;

mod msg;
mod msg_handler;

pub(super) async fn recv_ws(
    mut receiver: futures::stream::SplitStream<WebSocket>,
    mut presence: Presence,
    state: Arc<AppState>,
    id: i64,
    tx: broadcast::Sender<Broadcast>,
    room_id: crate::model::room::Id,
) {
    let mut dedup_ids = Vec::new();

    // Check if already authenticated
    if presence.session.is_some() {
        debug!("Sending authentication success message to client {}", id);

        let msg = BroadcastMsg {
            target: broadcast_msg::Target::One(id),
            content: super::ServerMsg::Authenticate {
                success: true,
                presence_id: presence.id.to_string(),
            },
        };

        if let Err(err) = tx.send(msg) {
            debug!("Failed to send authenticated message: {}", err);
        }
    } else {
        debug!("Client {} is not authenticated", id);
    }

    // Send join message
    let msg = BroadcastMsg {
        target: broadcast_msg::Target::All,
        content: super::ServerMsg::Join(presence.clone()),
    };

    if let Err(err) = tx.send(msg) {
        debug!("Failed to send join message: {}", err);
    }

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

        let msg_responses = msg_handler::handle_message(
            msg,
            &mut presence,
            &mut dedup_ids,
            state.clone(),
            &room_id,
        )
        .await;

        match msg_responses {
            Some(msg_responses) => {
                for response in msg_responses {
                    match response {
                        HandlerResult::Reply(msg) => {
                            debug!("sending message to {}", id);
                            trace!("sending message to {}: {:?}", id, msg); // Trace because logging the whole message is too verbose
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
                            trace!("broadcasting message: {:?}", msg); // Trace because logging the whole message is too verbose
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

    // Send leave message
    let msg = BroadcastMsg {
        target: broadcast_msg::Target::All,
        content: super::ServerMsg::Leave(presence),
    };

    if let Err(err) = tx.send(msg) {
        debug!("Failed to send leave message: {}", err);
    }

    debug!("Client {} disconnected", id);
}
