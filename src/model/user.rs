use serde_with::DisplayFromStr;

use super::Snowflake;

pub type Id = Snowflake;

#[serde_with::serde_as]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    #[serde_as(as = "DisplayFromStr")]
    pub id: Id,
    pub name: String,
    #[serde(skip)] // Don't expose (hashed) password to client
    pub password: String,
}
