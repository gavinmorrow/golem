use std::sync::Arc;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use log::{debug, trace};

use crate::model::AppState;

pub fn router() -> Router<Arc<AppState>> {
    trace!("ws router");
    Router::<Arc<AppState>>::new().route("/", get(handler))
}

#[debug_handler]
async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    trace!("ws connection requested");
    ws.on_upgrade(|ws| handle_ws(ws, state))
}

async fn handle_ws(mut ws: WebSocket, state: Arc<AppState>) {
    trace!("ws connection opened");
    while let Some(msg) = ws.recv().await {
        let Ok(msg) = msg else {
            // client disconnected
            break;
        };

        debug!("ws message: {:?}", msg);

        if ws.send(msg).await.is_err() {
            // client disconnected
            break;
        }
    }
    trace!("ws connection closed");
}
