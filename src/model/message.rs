pub struct Message {
	pub id: Snowflake,
	pub author: User,
	pub parent: Option<Snowflake>,
	pub content: String,
}
