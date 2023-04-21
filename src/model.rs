use tokio::sync::Mutex;

pub mod database;
pub mod message;
pub mod session;
pub mod user;

pub use database::Database;
pub use message::Message;
pub use session::Session;
pub use user::User;

type Snowflake = snowcloud::Snowflake<43, 8, 12>;

pub struct AppState {
    pub snowcloud: crate::Snowcloud,
    pub database: Mutex<Database>,
}

impl AppState {
    pub fn new() -> AppState {
        let snowcloud = crate::Snowcloud::new(crate::PRIMARY_ID, crate::EPOCH)
            .expect("Failed to create snowcloud.");
        let database = Mutex::new(Database::build().unwrap());

        AppState {
            snowcloud,
            database,
        }
    }
}
