use errors::TicketsResult;
use events::adapter::AdapterConfig;
use redis::Client;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

use events::PublishedMessage;

use crate::adapter::TicketsWebsocketAdapter;

pub struct RedisWebsocketAdapter;

impl TicketsWebsocketAdapter for RedisWebsocketAdapter {
    fn create_adapter(
        config: &AdapterConfig,
    ) -> TicketsResult<(
        JoinHandle<TicketsResult<()>>,
        UnboundedReceiver<PublishedMessage>,
    )> {
        log::info!("Redis adapter initializing.");

        let config = config.redis.as_ref().expect("Failed to read redis config.");
        let redis_client = Client::open(config.url.to_string())?;
        let mut conn = redis_client.get_connection()?;

        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

        Ok((
            tokio::spawn(async move {
                let mut pub_sub = conn.as_pubsub();
                pub_sub.subscribe(events::TICKETS_LIVE_EVENTS_CHANNEL)?;

                log::info!("Redis adapter looping...");

                loop {
                    let msg = pub_sub.get_message()?;
                    let payload: String = msg.get_payload()?;
                    let message: PublishedMessage = serde_json::from_str(&payload)?;
                    if sender.send(message).is_err() {
                        break;
                    }
                }
                Ok(())
            }),
            receiver,
        ))
    }
}
