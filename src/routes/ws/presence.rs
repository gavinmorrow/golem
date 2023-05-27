use crate::model::Snowflake;

pub(super) struct Presence {
    pub(super) id: Snowflake,
    pub(super) session_id: crate::model::session::Id,
    pub(super) name: String,
    pub(super) user: Option<crate::model::user::Id>,
}
