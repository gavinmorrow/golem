use axum::extract::ws::{self, Message::Text};

#[derive(Clone, Debug, serde::Deserialize)]
/// Basically just a [`Message`](crate::model::Message) without an id.
pub struct SendMessage {
    // pub author: crate::model::user::Id,
    pub parent: crate::model::message::Id,
    pub content: String,
    #[serde(default)]
    pub dedup_id: Option<String>,
}

#[derive(Clone, serde::Deserialize)]
pub struct PartialUser {
    pub name: String,
    /// The (**unhashed**) password
    pub password: String,
}

impl core::fmt::Debug for PartialUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Don't print the password
        f.debug_struct("PartialUser")
            .field("name", &self.name)
            .finish()
    }
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
    },
    ChangeName(String),
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
