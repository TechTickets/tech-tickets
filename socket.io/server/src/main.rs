use axum::Router;
use std::future::IntoFuture;
use std::sync::Arc;

use auth::jwt::JwtKeyPathsConfig;
use dry::config::load_config;
use events::adapter::{Adapter, AdapterConfig};

use crate::adapter::TicketsWebsocketAdapter;

mod adapter;
mod websocket;

#[derive(serde::Deserialize)]
struct Config {
    adapter_config: AdapterConfig,
    jwt: JwtKeyPathsConfig,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config: Config = load_config().expect("Failed to load config");

    let jwt = config.jwt.try_into().expect("Failed to load jwt config");

    log::info!("Initiating websocket server.");

    let (adapter_handle, message_pipe) = match config.adapter_config.adapter_type {
        #[cfg(feature = "redis_adapter")]
        Adapter::Redis => {
            adapter::redis::RedisWebsocketAdapter::create_adapter(&config.adapter_config)
                .expect("Could not create redis adapter.")
        }
    };

    let (message_receiver_handle, websocket_layer) =
        websocket::setup_server(Arc::new(jwt), message_pipe);

    let router = Router::new().layer(websocket_layer);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind port.");

    tokio::select! {
        _ = adapter_handle => (),
        _ = message_receiver_handle => (),
        _ = axum::serve(listener, router).into_future() => (),
    }
}
