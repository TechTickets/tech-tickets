use crate::channels::ChannelPurpose;
use crate::commands::{
    setup_application_commands, setup_default_commands, setup_management_commands, ParsedCommand,
};
use crate::guilds::GuildPurpose;
use crate::{respond, SharedAppState};
use errors::{ParsingError, TicketsError, TicketsResult};
use serenity::all::{
    ChannelId, CommandInteraction, ComponentInteraction, GuildId, Http, Interaction,
    ModalInteraction, PingInteraction, Ready, ResumedEvent,
};
use serenity::prelude::{Context, EventHandler};
use socketio_client::TicketsWebsocketClientExt;
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use uuid::Uuid;

pub struct AppState {
    #[allow(dead_code)]
    pub(super) app_changes_client: socketio_client::Client,
    #[allow(dead_code)]
    pub(super) ticket_event_client: socketio_client::Client,
    pub(super) shared_state: SharedAppState,
    pub(super) pg_pool: Pool<Postgres>,
}

impl AppState {
    async fn init_guild(
        &self,
        http: &Http,
        guild_id: GuildId,
        purpose: GuildPurpose,
        _app_id: Uuid,
    ) -> TicketsResult<()> {
        setup_application_commands(http, guild_id).await?;

        match purpose {
            GuildPurpose::Management => {
                setup_management_commands(http, guild_id).await?;
                // todo ensure roles
                // todo retrieve existing channels
                // todo retrieve app info
                // todo ensure embed information
                // todo relay missing messages
            }
            GuildPurpose::Consumer => {
                // todo retrieve existing channels
                // todo retrieve app categories
                // todo ensure consumer channels
                // todo ensure embed information
                // todo relay missing messages
            }
        }

        Ok(())
    }
}

#[serenity::async_trait]
impl EventHandler for AppState {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.shared_state.set_http(ctx.http.clone()).await;

        let managed_guild_ids = ready.guilds.iter().map(|item| item.id.get() as i64);
        let managed_guild_ids = managed_guild_ids.collect::<Vec<i64>>();

        fn flatten_query<
            Id: From<u64>,
            Purpose: TryFrom<String, Error = ParsingError>,
            Iter: Iterator<Item = (i64, String, Uuid)>,
        >(
            iter: Iter,
        ) -> Vec<(Id, Purpose, Uuid)> {
            iter.filter_map(|item| {
                Purpose::try_from(item.1)
                    .map(move |purpose| (Id::from(item.0 as u64), purpose, item.2))
                    .ok()
            })
            .collect()
        }

        let guild_data = sqlx::query!(
            "SELECT guild_id, purpose, app_id FROM discord_guilds WHERE guild_id = Any($1)",
            &managed_guild_ids
        )
        .fetch_all(&self.pg_pool)
        .await;

        let guild_data = flatten_query(guild_data.into_iter().flat_map(|records| {
            records
                .into_iter()
                .map(|record| (record.guild_id, record.purpose, record.app_id))
        }));

        self.shared_state
            .guild_cache
            .populate(guild_data.iter())
            .await;

        let channel_data = sqlx::query!(
            "SELECT id, purpose, app_id FROM discord_app_channels WHERE guild_id = Any($1)",
            &managed_guild_ids
        )
        .fetch_all(&self.pg_pool)
        .await;

        let channel_data = flatten_query(channel_data.into_iter().flat_map(|records| {
            records
                .into_iter()
                .map(|record| (record.id, record.purpose, record.app_id))
        }));

        self.shared_state
            .channel_cache
            .populate(channel_data.iter())
            .await;

        for guild_id in managed_guild_ids {
            if let Err(err) =
                setup_default_commands(&ctx.http, GuildId::from(guild_id as u64)).await
            {
                log::error!(
                    "Error setting up default commands for guild {}: {}",
                    guild_id,
                    err
                );
            }
        }

        for (guild_id, purpose, app_id) in guild_data.iter().cloned() {
            if let Err(err) = self.init_guild(&ctx.http, guild_id, purpose, app_id).await {
                log::error!(
                    "Error initializing guild ({}, {}, {}): {}",
                    guild_id,
                    purpose,
                    app_id,
                    err
                );
            }
        }

        let listeners = guild_data.iter().fold(
            HashSet::with_capacity(guild_data.len()),
            |mut listeners, (_, _, app_id)| {
                listeners.insert(app_id);
                listeners
            },
        );

        for listener in listeners {
            if let Err(err) = self.app_changes_client.listen_to(*listener, None).await {
                log::error!("Error listening to app changes for {listener}: {err}")
            }

            if let Err(err) = self.ticket_event_client.listen_to(*listener, None).await {
                log::error!("Error listening to ticket events for {listener}: {err}")
            }
        }
    }

    async fn resume(&self, ctx: Context, _: ResumedEvent) {
        self.shared_state.set_http(ctx.http.clone()).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let http = ctx.http.clone();
        if let Err((err, interaction_id, token)) = match interaction {
            Interaction::Command(interaction) => {
                let id = interaction.id;
                let token = interaction.token.clone();
                {
                    ParsedCommand::new(self.shared_state.clone(), ctx, interaction)
                        .execute()
                        .await
                }
                .map_err(move |err: TicketsError| (err, id, token))
            }
            Interaction::Modal(ModalInteraction { id, token, .. }) => {
                { Ok(()) }.map_err(move |err: TicketsError| (err, id, token))
            }
            Interaction::Ping(PingInteraction { id, token, .. }) => {
                { Ok(()) }.map_err(move |err: TicketsError| (err, id, token))
            }
            Interaction::Autocomplete(CommandInteraction { id, token, .. }) => {
                { Ok(()) }.map_err(move |err: TicketsError| (err, id, token))
            }
            Interaction::Component(ComponentInteraction { id, token, .. }) => {
                { Ok(()) }.map_err(move |err: TicketsError| (err, id, token))
            }
            _ => Ok(()),
        } {
            if let Err(err) = respond!(
                &http,
                interaction_id,
                &token,
                message {
                    ephemeral(true)
                    content(format!("Error in interaction. {}", err))
                }
            ) {
                log::error!("Failed to send error response: {}", err);
            }
        }
    }
}
