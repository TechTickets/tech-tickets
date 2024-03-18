#![feature(variant_count)]

use std::sync::Arc;
use std::time::Duration;

use serenity::all::GatewayIntents;

use app::AppState;
use auth::jwt::{JwtAccessor, JwtConfig, JwtData, JwtKeyPathsConfig};
use errors::TicketsResult;
use socketio_client::{AppChangesNamespace, TicketNamespace, TicketSocketConfig};

use crate::shared_state::SharedAppState;

mod app;
mod cache;
mod channels;
mod commands;
mod guilds;
mod interactions;
mod realtime;
mod shared_state;
mod modals;

#[derive(serde::Deserialize)]
pub struct DiscordTicketsConfig {
    collector_url: String,
    realtime_events_url: Option<String>,
    jwt: JwtKeyPathsConfig,
}

#[tokio::main]
async fn main() -> TicketsResult<()> {
    tracing_subscriber::fmt::init();

    let config: DiscordTicketsConfig = dry::config::load_config()?;

    let pg_pool = dry::database::connect().await?;
    sqlx::migrate!().set_locking(false).run(&pg_pool).await?;

    let jwt_config: JwtConfig = config.jwt.try_into()?;
    let jwt_config: Arc<JwtConfig> = Arc::new(jwt_config);

    let socket_config = TicketSocketConfig {
        server_url: config
            .realtime_events_url
            .unwrap_or_else(|| config.collector_url.to_string()),
        token: jwt_config
            .generate(
                JwtData {
                    accessor: JwtAccessor::DiscordSystem,
                },
                // only needs to be valid during the time of authentication
                // the system will run re-authentication requests for updated
                // token claim security requirements
                Duration::from_secs(60),
            )?
            .0,
    };

    let (app_changes_client, app_changes_receiver) =
        socketio_client::connect::<AppChangesNamespace>(&socket_config).await?;

    let (ticket_event_client, ticket_event_receiver) =
        socketio_client::connect::<TicketNamespace>(&socket_config).await?;

    let shared_app_state: SharedAppState = Default::default();

    let mut discord_client: serenity::Client = {
        let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

        let intents = GatewayIntents::GUILD_MEMBERS | GatewayIntents::DIRECT_MESSAGES;

        let app_state = AppState {
            app_changes_client,
            ticket_event_client,
            shared_state: shared_app_state.clone(),
            pg_pool,
        };

        serenity::Client::builder(&token, intents)
            .event_handler(app_state)
            .await
            .expect("Error creating client")
    };

    let realtime_app_changes =
        realtime::app_changes::read_app_changes(app_changes_receiver, shared_app_state.clone());

    let realtime_ticket_events = realtime::ticket_events::read_ticket_events(
        ticket_event_receiver,
        shared_app_state.clone(),
    );

    tokio::select! {
        res = realtime_app_changes => res??,
        res = realtime_ticket_events => res??,
        res = discord_client.start() => res?,
    }

    Ok(())
}
