use std::sync::Arc;

use log::error;

use crate::model::Snowflake;
use crate::{auth, model::Message};

use super::super::{AppState, ServerMsg, Session};
use super::msg::{ClientMsg, PartialUser, SendMessage};

pub(super) enum HandlerResult {
    Continue,
    Reply(ServerMsg),
    Broadcast(ServerMsg),
}

use HandlerResult::*;

pub(super) async fn handle_message(
    msg: ClientMsg,
    session: &mut Option<Session>,
    state: Arc<AppState>,
) -> HandlerResult {
    match msg {
        ClientMsg::AuthenticateToken(token) => authenticate_token(&state, token, session).await,
        ClientMsg::Authenticate(user) => authenticate(&state, user, session).await,
        ClientMsg::Pong => Continue,
        ClientMsg::Message(send_message) => message(&state, session, send_message).await,
        ClientMsg::LoadAllMessages => load_all_messages(&state).await,
        ClientMsg::LoadMessages { before, amount } => load_messages(&state, before, amount).await,
        ClientMsg::LoadChildren { parent, depth } => load_children(state, parent, depth).await,
    }
}

async fn authenticate_token(
    state: &Arc<AppState>,
    token: u64,
    session: &mut Option<Session>,
) -> HandlerResult {
    let database = state.database.lock().await;
    let Ok(session_db) = auth::verify_session(token, database) else {
				// Client sent invalid token
				return Reply(ServerMsg::Authenticate { success: false });
			};
    *session = Some(session_db);
    Reply(ServerMsg::Authenticate { success: true })
}

async fn authenticate(
    state: &Arc<AppState>,
    user: PartialUser,
    session: &mut Option<Session>,
) -> HandlerResult {
    let database = state.database.lock().await;
    let user_db = match database.get_user_by_name(&user.name) {
        Ok(Some(user)) => user,
        Ok(None) => {
            // User doesn't exist
            return Reply(ServerMsg::Authenticate { success: false });
        }
        Err(err) => {
            error!("Failed to get user from database: {}", err);
            return Reply(ServerMsg::Error);
        }
    };
    let success = auth::hash::check_passwords(user.password, user_db.password);
    *session = Some(Session::generate(state.next_snowflake(), user_db.id));
    Reply(ServerMsg::Authenticate { success })
}

async fn message(
    state: &Arc<AppState>,
    session: &mut Option<Session>,
    message: SendMessage,
) -> HandlerResult {
    let id = state.next_snowflake();
    if session.is_none() {
        return Reply(ServerMsg::Error);
    }
    let author = session.clone().unwrap().user_id;
    let message = Message {
        id,
        author,
        parent: message.parent,
        content: message.content,
    };
    let database = state.database.lock().await;
    match database.add_message(&message) {
        Ok(()) => Broadcast(ServerMsg::NewMessage(message)),
        Err(err) => {
            error!("Failed to add message to database: {:?}", err);
            return Reply(ServerMsg::Error);
        }
    }
}

async fn load_all_messages(state: &Arc<AppState>) -> HandlerResult {
    let database = state.database.lock().await;
    match database.get_messages() {
        Ok(messages) => Reply(ServerMsg::Messages(messages)),
        Err(err) => {
            error!("Failed to get messages from database: {:?}", err);
            Reply(ServerMsg::Error)
        }
    }
}

async fn load_messages(
    state: &Arc<AppState>,
    before: Option<Snowflake>,
    amount: u8,
) -> HandlerResult {
    let database = state.database.lock().await;
    match database.get_some_messages(before, amount) {
        Ok(messages) => Reply(ServerMsg::Messages(messages)),
        Err(err) => {
            error!("Failed to get messages from database: {:?}", err);
            Reply(ServerMsg::Error)
        }
    }
}

async fn load_children(state: Arc<AppState>, parent: Snowflake, depth: u8) -> HandlerResult {
    let database = state.database.lock().await;
    match database.get_children_of(Some(&parent), depth) {
        Ok(messages) => Reply(ServerMsg::Messages(messages)),
        Err(err) => {
            error!("Failed to get messages from database: {:?}", err);
            Reply(ServerMsg::Error)
        }
    }
}
