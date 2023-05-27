use crate::model::Snowflake;

pub(super) struct Session {
    pub(super) id: Snowflake,
    pub(super) name: String,
    pub(super) user: Option<crate::model::user::Id>,
}
