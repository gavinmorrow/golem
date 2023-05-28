use serde_with::{serde_as, DisplayFromStr};

use crate::model::{Session, Snowflake};

#[serde_as]
#[derive(Clone, Debug, serde::Serialize)]
pub struct Presence {
    #[serde_as(as = "DisplayFromStr")]
    pub id: Snowflake,
    pub session: Option<Session>,
    pub name: String,
}
