use adapter::{redis::RedisWebsocketAdapter, TicketsWebsocketAdapter};
use auth::jwt::JwtConfig;
use errors::TicketsResult;
use events::adapter::{Adapter, AdapterConfig};
use socketioxide::layer::SocketIoLayer;
use std::sync::Arc;
use tokio::task::JoinHandle;

mod adapter;
mod websocket;

pub async fn setup_websocket_layer(
    adapter_config: &AdapterConfig,
    jwt: Arc<JwtConfig>,
) -> TicketsResult<(
    JoinHandle<TicketsResult<()>>,
    JoinHandle<TicketsResult<()>>,
    SocketIoLayer,
)> {
    let (adapter_handle, message_pipe) = match adapter_config.adapter_type {
        #[cfg(feature = "redis_adapter")]
        Adapter::Redis => RedisWebsocketAdapter::create_adapter(adapter_config)?,
    };

    let (message_receiver_handle, websocket_layer) = websocket::setup_server(jwt, message_pipe);

    Ok((adapter_handle, message_receiver_handle, websocket_layer))
}
