use super::Snowflake;

pub type Id = Snowflake;

pub struct Message {
	pub id: Id,
	pub author: super::user::Id,
	pub parent: Option<Id>,
	pub content: String,
}
