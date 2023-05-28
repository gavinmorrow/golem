use crate::auth;

use super::Snowflake;

pub type Id = Snowflake;
pub type Token = i64;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Session {
    pub id: Id,
    #[serde(skip)] // Don't expose token to client
    pub token: Token,
    pub user_id: super::user::Id,
}

impl Session {
    pub fn new(id: Id, token: Token, user_id: super::user::Id) -> Session {
        Session { id, token, user_id }
    }

    pub fn generate(id: Id, user_id: super::user::Id) -> Session {
        // Generate token
        let token = auth::token::generate_token();

        Session { id, token, user_id }
    }
}
