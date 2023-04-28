use log::{debug, error};
use tokio::sync::MutexGuard;

use crate::model::{Database, Session};

pub enum Error {
    SessionNotFound,
    DatabaseError,
}

pub fn verify_session(token: u64, database: MutexGuard<Database>) -> Result<Session, Error> {
    // Get and verify session
    match database.get_session_from_token(&token) {
        Ok(Some(session)) => return Ok(session),
        Ok(None) => {
            debug!("Session {} not found in database", token);
            return Err(Error::SessionNotFound);
        }
        Err(err) => {
            error!("Failed to get session from database: {}", err);
            return Err(Error::DatabaseError);
        }
    };
}
