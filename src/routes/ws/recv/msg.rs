use axum::extract::ws::{self, Message::Text};
use serde_with::DisplayFromStr;

#[serde_with::serde_as]
#[derive(Clone, Debug, serde::Deserialize)]
/// Basically just a [`Message`](crate::model::Message) without an id.
pub struct SendMessage {
    // pub author: crate::model::user::Id,
    #[serde_as(as = "DisplayFromStr")]
    pub parent: crate::model::message::Id,
    pub content: String,
    #[serde(default)]
    pub dedup_id: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct PartialUser {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum ClientMsg {
    Authenticate(PartialUser),
    Pong,
    Message(SendMessage),
    LoadAllMessages,
    LoadMessages {
        before: Option<crate::model::message::Id>,
        amount: u8,
    },
    LoadChildren {
        parent: crate::model::message::Id,
        depth: u8,
    },
}

impl ClientMsg {
    /// Build a [`ClientMsg`] from a [`ws::Message`].
    /// The message must be the [`Text`](ws::Message::Text) variant.
    pub fn build(msg: ws::Message) -> Result<ClientMsg, BuildError> {
        let Text(msg) = msg else {
            return Err(BuildError::MsgType);
        };

        match serde_json::from_str(&msg) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(BuildError::Serde(err)),
        }
    }
}

#[derive(Debug)]
pub enum BuildError {
    MsgType,
    Serde(serde_json::error::Error),
}
