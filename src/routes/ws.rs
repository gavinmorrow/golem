use std::sync::Arc;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use axum_macros::debug_handler;

use crate::model::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new().route("/", get(handler))
}

#[debug_handler]
async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|ws| handle_ws(ws, state))
}

async fn handle_ws(mut ws: WebSocket, state: Arc<AppState>) {
	while let Some(msg) = ws.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if ws.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}
