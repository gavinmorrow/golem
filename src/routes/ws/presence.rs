use crate::model::{Session, Snowflake};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Presence {
    pub id: Snowflake,
    pub session: Option<Session>,
    pub name: String,
}
