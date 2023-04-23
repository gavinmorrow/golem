use super::Snowflake;

pub type Id = Snowflake;

pub struct Session {
    pub id: Id,
    pub user_id: super::user::Id,
}

impl Session {
    pub fn new(session_id: Id, user_id: super::user::Id) -> Session {
        Session { id: session_id, user_id }
    }
}
