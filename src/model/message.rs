use super::{user, Snowflake};

pub type Id = Snowflake;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: Id,
    pub author: user::Id,
    pub parent: Option<Id>,
    pub content: String,
}
