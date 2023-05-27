use crate::model::{Session, Snowflake};

#[derive(Clone)]
pub(super) struct Presence {
    pub(super) id: Snowflake,
    pub(super) session: Option<Session>,
    pub(super) name: String,
}
