use super::Snowflake;

pub type Token = u64;

pub struct Session {
    pub token: Token,
    pub user_id: super::user::Id,
}

impl Session {
    pub fn new(session_token: Token, user_id: super::user::Id) -> Session {
        Session {
            token: session_token,
            user_id,
        }
    }
}
