use super::Snowflake;

pub type Id = Snowflake;

#[derive(Debug, serde::Serialize)]
pub struct User {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct PartialUser {
    pub name: String,
}
