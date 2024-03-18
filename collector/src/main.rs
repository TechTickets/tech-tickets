use std::sync::Arc;

use axum::extract::FromRef;
use axum::Router;

use auth::jwt::{JwtConfig, JwtKeyPathsConfig};
use dry::config::load_config;
use errors::TicketsResult;
use events::adapter::Adapter;

use crate::state::GlobalState;

mod axum_ext;
mod consumer;
mod staff;
mod state;

#[derive(serde::Deserialize)]
pub struct TicketCollectorConfig {
    #[serde(rename = "adapter")]
    pub adapter_config: events::adapter::AdapterConfig,
    pub jwt: JwtKeyPathsConfig,
}
impl FromRef<GlobalState> for Arc<JwtConfig> {
    fn from_ref(input: &GlobalState) -> Self {
        input.jwt_config.clone()
    }
}

#[tokio::main]
async fn main() -> TicketsResult<()> {
    tracing_subscriber::fmt::init();

    let config: TicketCollectorConfig = load_config()?;

    let pg_client = dry::database::connect().await?;
    sqlx::migrate!().set_locking(false).run(&pg_client).await?;

    let adapter = match config.adapter_config.adapter_type {
        Adapter::Redis => {
            let config = config
                .adapter_config
                .redis
                .as_ref()
                .expect("Could not find redis config");
            redis::Client::open(config.url.to_string())?
        }
    };

    let state = GlobalState {
        pg_client,
        jwt_config: Arc::new(config.jwt.try_into()?),
        emitter: Arc::new(adapter),
    };

    let app = Router::new();

    let app = consumer::extend_router(app);
    let app = staff::extend_router(app);

    if cfg!(feature = "nest-websocket-server") {
        #[cfg(feature = "nest-websocket-server")]
        {
            let (adapter_handle, message_handle, socket_io_layer) =
                socketio_server::setup_websocket_layer(
                    &config.adapter_config,
                    state.jwt_config.clone(),
                )
                .await?;

            let app = app.with_state(state).layer(socket_io_layer);

            let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

            use std::future::IntoFuture;

            log::info!("Executing rest API and websocket server on :8000");

            Ok(tokio::select! {
                err = adapter_handle => err??,
                err = message_handle => err??,
                err = axum::serve(listener, app).into_future() => err?,
            })
        }

        #[cfg(not(feature = "nest-websocket-server"))]
        unreachable!()
    } else {
        let app = app.with_state(state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

        log::info!("Executing rest API server on :8000");

        Ok(axum::serve(listener, app).await?)
    }
}
