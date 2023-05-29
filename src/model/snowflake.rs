use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::de::Error;

type InnerSnowflake = snowcloud::Snowflake<43, 8, 12>;

#[derive(Clone, Debug, PartialEq, Eq)]
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

impl serde::Serialize for Snowflake {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.id().to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Snowflake {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let num = String::deserialize(deserializer)?;
        Ok(Snowflake::from_str(&num).map_err(D::Error::custom)?)
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Default implementation just delegates to `deserialize` impl.
        *place = serde::Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}
