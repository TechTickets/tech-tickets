use errors::TicketsResult;
use events::adapter::AdapterConfig;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

use events::PublishedMessage;

#[cfg(feature = "redis_adapter")]
pub mod redis;

pub trait TicketsWebsocketAdapter {
    fn create_adapter(
        config: &AdapterConfig,
    ) -> TicketsResult<(
        JoinHandle<TicketsResult<()>>,
        UnboundedReceiver<PublishedMessage>,
    )>;
}
