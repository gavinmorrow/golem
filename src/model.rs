use tokio::sync::Mutex;

pub mod database;
pub mod message;
pub mod user;

pub use database::Database;
pub use message::Message;
pub use user::User;

type Snowflake = snowcloud::Snowflake<43, 8, 12>;

pub struct AppState {
    pub snowcloud: crate::Snowcloud,
    pub database: Mutex<Database>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            snowcloud: crate::Snowcloud::new(crate::PRIMARY_ID, crate::EPOCH)
                .expect("Failed to create snowcloud."),
            database: Mutex::new(Database::new()),
        }
    }
}
