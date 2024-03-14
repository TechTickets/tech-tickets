#![feature(variant_count)]
#![feature(const_trait_impl)]
#![feature(effects)]

use std::path::PathBuf;

use crate::state::DiscordAppState;
use serenity::prelude::GatewayIntents;
use tickets_common::config::load_config;
use tickets_common::jwt::{JwtConfig, JwtKeyPathsConfig};

mod cache;
mod commands;
mod events;
mod interactions;
mod modals;
mod state;
mod websocket;

#[derive(serde::Deserialize)]
struct DiscordTicketsConfig {
    ticket_collector_base_url: String,
    jwt_keys: JwtKeyPathsConfig,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config: DiscordTicketsConfig = load_config()?;

    let jwt_config: JwtConfig = config.jwt_keys.try_into()?;

    let pg_client = tickets_common::database::connect().await?;
    sqlx::migrate!().set_locking(false).run(&pg_client).await?;

    let (app_state, _shared_state) =
        DiscordAppState::create(config.ticket_collector_base_url, pg_client, jwt_config)?;

    let mut discord_client: serenity::Client = {
        let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

        let intents = GatewayIntents::GUILD_MEMBERS;

        serenity::Client::builder(&token, intents)
            .event_handler(app_state)
            .await
            .expect("Error creating client")
    };

    if let Err(why) = discord_client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}
