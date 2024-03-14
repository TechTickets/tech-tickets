use std::sync::Arc;

use serenity::all::{ChannelId, Context, CreateChannel, GuildId, Http};
use sqlx::{Pool, Postgres};
use tickets_common::jwt::{JwtAccessor, JwtConfig};
use tickets_common::requests::sdk::{InternalSdk, SignedTicketClient};

use crate::cache::channels::{ChannelCache, ChannelPurpose};
use crate::cache::guilds::GuildCache;
use crate::cache::roles::RolesCache;
use crate::cache::users::UsersCache;

// state which is shared between the discord and web apps
pub struct SharedAppState {
    pub guild_cache: GuildCache,
    pub user_cache: UsersCache,
    pub roles_cache: RolesCache,
    pub channel_cache: ChannelCache,
}

pub struct DiscordAppState {
    pub postgres_client: Arc<Pool<Postgres>>,
    pub internal_sdk: InternalSdk,
    // main authed client
    pub ticket_system_client: SignedTicketClient,
    // shared
    pub shared_state: Arc<SharedAppState>,
}

impl DiscordAppState {
    pub(crate) fn create<S: Into<String>>(
        base_url: S,
        postgres_client: Pool<Postgres>,
        jwt_config: JwtConfig,
    ) -> anyhow::Result<(Self, Arc<SharedAppState>)> {
        let shared_app_state = Arc::new(SharedAppState {
            guild_cache: GuildCache::default(),
            user_cache: UsersCache::default(),
            roles_cache: RolesCache::default(),
            channel_cache: ChannelCache::default(),
        });
        let internal_sdk: InternalSdk = (base_url.into(), jwt_config, "discord").try_into()?;
        Ok((
            Self {
                postgres_client: Arc::new(postgres_client),
                ticket_system_client: internal_sdk
                    .sign_client(JwtAccessor::DiscordSystem, InternalSdk::DEFAULT_TTL)?,
                internal_sdk,
                shared_state: shared_app_state.clone(),
            },
            shared_app_state,
        ))
    }

    pub async fn create_channel_for_guild(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        purpose: ChannelPurpose,
        channel_spec: impl FnOnce(CreateChannel) -> CreateChannel,
    ) -> anyhow::Result<ChannelId> {
        let app_id = self
            .shared_state
            .guild_cache
            .get_app_id(guild_id)
            .await
            .ok_or_else(|| anyhow::format_err!("Could not find associated app for guild."))?;

        let channel_name = purpose.channel_name();

        let channel = guild_id
            .create_channel(ctx, channel_spec(CreateChannel::new(channel_name)))
            .await?;

        sqlx::query!(
            "INSERT INTO discord_app_channels (id, guild_id, app_id, purpose) VALUES ($1, $2, $3, $4)",
            channel.id.get() as i64,
            guild_id.get() as i64,
            app_id,
            purpose.to_string()
        ).execute(self.postgres_client.as_ref()).await?;

        self.shared_state
            .channel_cache
            .insert(guild_id, purpose, channel.id)
            .await;
        Ok(channel.id)
    }
}

pub struct WebsocketAppState {
    pub discord_client: Arc<Http>,
    pub shared_state: Arc<SharedAppState>,
}

impl WebsocketAppState {
    pub fn create(discord_client: &Arc<Http>, shared_state: &Arc<SharedAppState>) -> Self {
        Self {
            discord_client: discord_client.clone(),
            shared_state: shared_state.clone(),
        }
    }
}
