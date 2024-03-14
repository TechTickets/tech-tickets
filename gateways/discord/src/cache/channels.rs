use bimap::BiMap;
use serenity::all::{ChannelId, GuildId};
use sqlx::{Pool, Postgres};
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
// more variants will come soon
#[allow(clippy::enum_variant_names)]
pub enum ChannelPurpose {
    ManagementCategory,
    ManagementBotCommands,
    ManagementInfo,
    StaffCategory,
    StaffBotCommands,
    StaffInfo,
}

impl ChannelPurpose {
    pub fn channel_name(self) -> &'static str {
        match self {
            ChannelPurpose::ManagementCategory => "Tickets Management",
            ChannelPurpose::ManagementBotCommands => "mgmt-bot-commands",
            ChannelPurpose::ManagementInfo => "mgmt-info",
            ChannelPurpose::StaffCategory => "Tech Tickets Staff",
            ChannelPurpose::StaffBotCommands => "bot-commands",
            ChannelPurpose::StaffInfo => "staff-info",
        }
    }
}

impl TryFrom<String> for ChannelPurpose {
    type Error = anyhow::Error;

    fn try_from(purpose: String) -> anyhow::Result<Self> {
        Ok(match purpose.as_str() {
            "mgmt-category" => ChannelPurpose::ManagementCategory,
            "mgmt-bot-commands" => ChannelPurpose::ManagementBotCommands,
            "mgmt-info" => ChannelPurpose::ManagementInfo,
            "staff-category" => ChannelPurpose::StaffCategory,
            "staff-bot-commands" => ChannelPurpose::StaffBotCommands,
            "staff-info" => ChannelPurpose::StaffInfo,
            _ => anyhow::bail!("Invalid channel purpose: {}", purpose),
        })
    }
}

impl Display for ChannelPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelPurpose::ManagementCategory => write!(f, "mgmt-category"),
            ChannelPurpose::ManagementBotCommands => write!(f, "mgmt-bot-commands"),
            ChannelPurpose::ManagementInfo => write!(f, "mgmt-info"),
            ChannelPurpose::StaffCategory => write!(f, "staff-category"),
            ChannelPurpose::StaffBotCommands => write!(f, "staff-bot-commands"),
            ChannelPurpose::StaffInfo => write!(f, "staff-info"),
        }
    }
}

pub struct ChannelCache {
    inner: Arc<RwLock<BiMap<(ChannelPurpose, GuildId), ChannelId>>>,
}

impl Default for ChannelCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BiMap::new())),
        }
    }
}

impl ChannelCache {
    pub async fn init_channel_cache(
        &self,
        pool: &Pool<Postgres>,
        guilds: impl Iterator<Item = GuildId>,
    ) -> anyhow::Result<()> {
        let managed_channels: BiMap<(ChannelPurpose, GuildId), ChannelId> = sqlx::query!(
            "SELECT purpose, id, guild_id FROM discord_app_channels WHERE guild_id = Any($1)",
            &guilds.map(|guild| guild.get() as i64).collect::<Vec<_>>()
        )
        .fetch_all(pool)
        .await?
        .iter()
        .filter_map(|value| {
            ChannelPurpose::try_from(value.purpose.to_string())
                .ok()
                .map(|purpose| {
                    (
                        (purpose, GuildId::from(value.guild_id as u64)),
                        ChannelId::from(value.id as u64),
                    )
                })
        })
        .collect();

        let mut inner = self.inner.write().await;
        *inner = managed_channels;

        Ok(())
    }

    pub async fn insert(&self, guild_id: GuildId, purpose: ChannelPurpose, channel_id: ChannelId) {
        let mut inner = self.inner.write().await;
        inner.insert((purpose, guild_id), channel_id);
    }
}
