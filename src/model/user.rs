use super::Snowflake;

pub type Id = Snowflake;

#[derive(Debug)]
pub struct User {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PartialUser {
    pub name: String,
}
