use bimap::BiMap;
use errors::{ParsingError, TicketsError};
use serenity::all::GuildId;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum GuildPurpose {
    Consumer,
    Management,
}

const CONSUMER_GUILD_PURPOSE: &str = "consumer";
const MANAGEMENT_GUILD_PURPOSE: &str = "management";

impl Display for GuildPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuildPurpose::Consumer => write!(f, "{}", CONSUMER_GUILD_PURPOSE),
            GuildPurpose::Management => write!(f, "{}", MANAGEMENT_GUILD_PURPOSE),
        }
    }
}

impl TryFrom<String> for GuildPurpose {
    type Error = ParsingError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(match s.as_str() {
            CONSUMER_GUILD_PURPOSE => GuildPurpose::Consumer,
            MANAGEMENT_GUILD_PURPOSE => GuildPurpose::Management,
            _ => Err(ParsingError::InvalidGuildPurpose(s))?,
        })
    }
}

pub type GuildCache = crate::cache::GenericIdCache<GuildId, GuildPurpose>;
