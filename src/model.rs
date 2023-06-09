use std::sync::Arc;

use tokio::sync::Mutex;

pub mod database;
pub mod message;
pub mod room;
pub mod session;
pub mod snowflake;
pub mod user;

pub use database::Database;
pub use message::Message;
pub use room::Room;
pub use session::Session;
pub use snowflake::Snowflake;
pub use user::User;

#[derive(Clone)]
pub struct AppState {
    pub snowcloud: crate::Snowcloud,
    pub database: Arc<Mutex<Database>>,
}

impl AppState {
    pub fn new() -> AppState {
        let snowcloud = crate::Snowcloud::new(crate::PRIMARY_ID, crate::EPOCH)
            .expect("Failed to create snowcloud.");
        let database = Arc::new(Mutex::new(Database::build().unwrap()));

        AppState {
            snowcloud,
            database,
        }
    }

    pub fn next_snowflake(&self) -> Snowflake {
        self.snowcloud.next_id().unwrap().into()
    }
}
