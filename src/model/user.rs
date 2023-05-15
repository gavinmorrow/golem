use super::Snowflake;

pub type Id = Snowflake;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Id,
    pub name: String,
    pub password: String,
}
