use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    sync::Arc,
};

use tokio::sync::Mutex;

pub mod database;
pub mod message;
pub mod session;
pub mod user;

pub use database::Database;
pub use message::Message;
pub use session::Session;
pub use user::User;

type InnerSnowflake = snowcloud::Snowflake<43, 8, 12>;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Snowflake(InnerSnowflake);

impl Snowflake {
    pub fn id(&self) -> i64 {
        self.0.id()
    }
}

impl TryFrom<i64> for Snowflake {
    type Error = snowcloud::Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Snowflake(InnerSnowflake::try_from(value)?))
    }
}

impl From<InnerSnowflake> for Snowflake {
    fn from(value: InnerSnowflake) -> Self {
        Snowflake(value)
    }
}

impl FromStr for Snowflake {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let num = s.parse::<i64>()?;
        Ok(Snowflake(InnerSnowflake::try_from(num)?))
    }
}

impl Display for Snowflake {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

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
