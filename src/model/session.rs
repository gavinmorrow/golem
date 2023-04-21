use super::Snowflake;

pub type Id = Snowflake;

pub struct Session {
    pub id: Id,
    pub user_id: super::user::Id,
}
