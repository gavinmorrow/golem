use std::sync::Arc;

use super::Sender;

use crate::model::AppState;

pub(super) struct WsState {
    pub(super) appstate: Arc<AppState>,
    pub(super) tx: Sender,
}
