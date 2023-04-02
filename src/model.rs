mod database;
mod message;
mod user;

use database::Database;
use message::Message;
use user::User;

type Snowflake = snowcloud::Snowflake<43, 8, 12>;
