use std::sync::Arc;

use tokio::sync::Mutex;

pub mod database;
pub mod message;
pub mod session;
pub mod user;

pub use database::Database;
pub use message::Message;
pub use session::Session;
pub use user::User;

pub type Snowflake = snowcloud::Snowflake<43, 8, 12>;

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
        self.snowcloud.next_id().unwrap()
    }
}
