use super::Snowflake;

pub type Id = Snowflake;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Id,
    pub name: String,
    #[serde(skip)] // Don't expose (hashed) password to client
    pub password: String,
}
