use super::{user, Snowflake};

pub type Id = Snowflake;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Eq)]
pub struct Message {
    pub id: Id,
    pub author: user::Id,
    pub author_name: String,
    pub parent: Id, /* The way that Golem expresses a top level message is
                     * by making the parent of said message the room id. */
    pub content: String,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

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
