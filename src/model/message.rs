use super::{user, Snowflake};
use serde_with::DisplayFromStr;

pub type Id = Snowflake;

#[serde_with::serde_as]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    #[serde_as(as = "DisplayFromStr")]
    pub id: Id,
    #[serde_as(as = "DisplayFromStr")]
    pub author: user::Id,
    pub author_name: String,
    #[serde_as(as = "DisplayFromStr")]
    pub parent: Id,
    pub content: String,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Message {}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.id().partial_cmp(&other.id.id())
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.id().cmp(&other.id.id())
    }
}
