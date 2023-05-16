use std::sync::Arc;

use log::{debug, error, trace};

use crate::model::Snowflake;
use crate::{auth, model::Message};

use super::super::{AppState, ServerMsg, Session};
use super::msg::{ClientMsg, PartialUser, SendMessage};

#[derive(Debug)]
pub(super) enum HandlerResult {
    Reply(ServerMsg),
    Broadcast(ServerMsg),
}

use HandlerResult::*;

type Response = Vec<HandlerResult>;

pub(super) async fn handle_message(
    msg: ClientMsg,
    session: &mut Option<Session>,
    mut dedup_ids: &mut Vec<Option<String>>,
    state: Arc<AppState>,
) -> Option<Response> {
    Some(match msg {
        ClientMsg::AuthenticateToken(token) => authenticate_token(&state, token, session).await,
        ClientMsg::Authenticate(user) => authenticate(&state, user, session).await,
        ClientMsg::Pong => return None,
        ClientMsg::Message(send_message) => {
            match session {
                Some(session) => {
                    message(&state, session.clone(), &mut dedup_ids, send_message).await
                }
                None => {
                    // Client isn't authenticated
                    trace!("Client attempted to send message without authenticating");
                    vec![Reply(ServerMsg::Unauthenticated)]
                }
            }
        }
        ClientMsg::LoadAllMessages => load_all_messages(&state).await,
        ClientMsg::LoadMessages { before, amount } => load_messages(&state, before, amount).await,
        ClientMsg::LoadChildren { parent, depth } => load_children(state, parent, depth).await,
    })
}

async fn authenticate_token(
    state: &Arc<AppState>,
    token: u64,
    session: &mut Option<Session>,
) -> Response {
    trace!("Authenticating user from token");

    let database = state.database.lock().await;
    let Ok(session_db) = auth::verify_session(token, database) else {
        // Client sent invalid token
        return vec![Reply(ServerMsg::Authenticate { success: false })];
    };

    let user_id = session_db.user_id.clone();
    *session = Some(session_db);

    // Don't need to drop database here because it was moved into the auth::verify_session function
    // and is dropped there.

    finish_authentication(state, user_id).await
}

async fn authenticate(
    state: &Arc<AppState>,
    user: PartialUser,
    session: &mut Option<Session>,
) -> Response {
    trace!("Authenticating user from credentials");

    let database = state.database.lock().await;
    let user_db = match database.get_user_by_name(&user.name) {
        Ok(Some(user)) => user,
        Ok(None) => {
            // User doesn't exist
            error!("User not found in database: {}", user.name);
            return vec![Reply(ServerMsg::Authenticate { success: false })];
        }
        Err(err) => {
            error!("Failed to get user from database: {}", err);
            return vec![Reply(ServerMsg::Error)];
        }
    };

    if !auth::hash::check_passwords(user.password, user_db.password) {
        // Password incorrect
        return vec![Reply(ServerMsg::Authenticate { success: false })];
    }

    let user_id = user_db.id.clone();
    *session = Some(Session::generate(state.next_snowflake(), user_db.id));

    drop(database); // Prevent deadlock

    finish_authentication(state, user_id).await
}

async fn finish_authentication(state: &Arc<AppState>, user_id: crate::model::user::Id) -> Response {
    // resolve user
    let mut user = match state.database.lock().await.get_user(&user_id) {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("User {:?} not found in database.", user_id);
            return vec![Reply(ServerMsg::Authenticate { success: false })];
        }
        Err(err) => {
            error!("Failed to get user from database: {}", err);
            return vec![Reply(ServerMsg::Error)];
        }
    };

    debug!("Finished authentication for user {:?}", user_id);

    user.password = String::new(); // Don't send password to client

    vec![
        Reply(ServerMsg::Authenticate { success: true }),
        Broadcast(ServerMsg::Join(user)),
    ]
}

async fn message(
    state: &Arc<AppState>,
    session: Session,
    dedup_ids: &mut Vec<Option<String>>,
    message: SendMessage,
) -> Response {
    debug!("Received message from client {}", session.id);

    let id = state.next_snowflake();

    let dedup_id = message.dedup_id.clone();
    if dedup_id.is_some() && dedup_ids.contains(&dedup_id) {
        // Message is a duplicate
        debug!(
            "Duplicate message detected: {:?} from client {}",
            dedup_id, session.id
        );
        return vec![Reply(ServerMsg::Duplicate(dedup_id.unwrap()))];
    }

    let author = session.user_id;
    let message = Message {
        id,
        author,
        parent: message.parent,
        content: message.content,
    };
    let database = state.database.lock().await;
    match database.add_message(&message) {
        Ok(()) => {
            dedup_ids.push(dedup_id);
            vec![Broadcast(ServerMsg::NewMessage(message))]
        }
        Err(err) => {
            error!("Failed to add message to database: {:?}", err);
            return vec![Reply(ServerMsg::Error)];
        }
    }
}

async fn load_all_messages(state: &Arc<AppState>) -> Response {
    trace!("Loading all messages");
    let database = state.database.lock().await;
    match database.get_messages() {
        Ok(messages) => vec![Reply(ServerMsg::Messages(messages))],
        Err(err) => {
            error!("Failed to get messages from database: {:?}", err);
            vec![Reply(ServerMsg::Error)]
        }
    }
}

async fn load_messages(state: &Arc<AppState>, before: Option<Snowflake>, amount: u8) -> Response {
    let database = state.database.lock().await;
    match database.get_some_messages(before, amount) {
        Ok(messages) => vec![Reply(ServerMsg::Messages(messages))],
        Err(err) => {
            error!("Failed to get messages from database: {:?}", err);
            vec![Reply(ServerMsg::Error)]
        }
    }
}

async fn load_children(state: Arc<AppState>, parent: Snowflake, depth: u8) -> Response {
    let database = state.database.lock().await;
    match database.get_children_of(Some(&parent), depth) {
        Ok(messages) => vec![Reply(ServerMsg::Messages(messages))],
        Err(err) => {
            error!("Failed to get messages from database: {:?}", err);
            vec![Reply(ServerMsg::Error)]
        }
    }
}
