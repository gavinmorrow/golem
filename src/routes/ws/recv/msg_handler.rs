use std::sync::Arc;
use std::vec;

use log::{debug, error, trace};

use crate::model::Snowflake;
use crate::routes::ws::presence::Presence;
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
    presence: &mut Presence,
    mut dedup_ids: &mut Vec<Option<String>>,
    state: Arc<AppState>,
    room_id: &crate::model::room::Id,
) -> Option<Response> {
    Some(match msg {
        ClientMsg::Authenticate(user) => authenticate(&state, user, presence).await,
        ClientMsg::Pong => return None,
        ClientMsg::Message(send_message) => {
            message(&state, presence.clone(), &mut dedup_ids, send_message).await
        }
        ClientMsg::LoadAllMessages => load_all_messages(&state, room_id).await,
        ClientMsg::LoadMessages { before, amount } => load_messages(&state, before, amount).await,
        ClientMsg::LoadChildren { parent } => load_children(state, parent).await,
        ClientMsg::ChangeName(name) => change_name(&state, presence, name).await,
    })
}

async fn authenticate(
    state: &Arc<AppState>,
    user: PartialUser,
    presence: &mut Presence,
) -> Response {
    trace!("Authenticating user from credentials");

    let presence_id = presence.id.to_string();

    let database = state.database.lock().await;
    let user_db = match database.get_user_by_name(&user.name) {
        Ok(Some(user)) => user,
        Ok(None) => {
            // User doesn't exist
            error!("User not found in database: {}", user.name);
            return vec![Reply(ServerMsg::Authenticate {
                success: false,
                presence_id,
            })];
        }
        Err(err) => {
            error!("Failed to get user from database: {}", err);
            return vec![Reply(ServerMsg::Error)];
        }
    };

    if !auth::hash::check_passwords(user.password, user_db.password) {
        // Password incorrect
        return vec![Reply(ServerMsg::Authenticate {
            success: false,
            presence_id,
        })];
    }

    presence.session = Some(Session::generate(state.next_snowflake(), user_db.id));
    presence.name = user.name;

    return vec![
        Broadcast(ServerMsg::Update(presence.clone())),
        Reply(ServerMsg::Authenticate {
            success: true,
            presence_id,
        }),
    ];
}

async fn message(
    state: &Arc<AppState>,
    presence: Presence,
    dedup_ids: &mut Vec<Option<String>>,
    message: SendMessage,
) -> Response {
    debug!("Received message from client {}", presence.id);

    let id = state.next_snowflake();

    let dedup_id = message.dedup_id.clone();
    if dedup_id.is_some() && dedup_ids.contains(&dedup_id) {
        // Message is a duplicate
        debug!(
            "Duplicate message detected: {:?} from client {}",
            dedup_id, presence.id
        );
        return vec![Reply(ServerMsg::Duplicate(dedup_id.unwrap()))];
    }

    let database = state.database.lock().await;

    let message = Message {
        id,
        author: presence.session.map(|s| s.user_id).unwrap_or(presence.id),
        author_name: presence.name,
        parent: message.parent,
        content: message.content,
    };

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

async fn load_all_messages(state: &Arc<AppState>, room_id: &crate::model::room::Id) -> Response {
    trace!("Loading all messages");
    let database = state.database.lock().await;
    match database.get_children_of(Some(room_id)) {
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

async fn load_children(state: Arc<AppState>, parent: Snowflake) -> Response {
    let database = state.database.lock().await;
    match database.get_children_of(Some(&parent)) {
        Ok(messages) => vec![Reply(ServerMsg::Messages(messages))],
        Err(err) => {
            error!("Failed to get messages from database: {:?}", err);
            vec![Reply(ServerMsg::Error)]
        }
    }
}

async fn change_name(_state: &AppState, presence: &mut Presence, name: String) -> Response {
    presence.name = name;

    vec![Reply(ServerMsg::Update(presence.clone()))]
}
