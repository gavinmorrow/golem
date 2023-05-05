use axum::extract::ws::{self, WebSocket};
use futures::SinkExt;
use log::debug;
use tokio::sync::broadcast;

use super::{broadcast_msg, Broadcast};

pub(super) async fn broadcast_handler(
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
