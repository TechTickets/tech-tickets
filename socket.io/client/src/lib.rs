use std::time::Duration;

use errors::TicketsResult;
pub use rust_socketio::asynchronous::Client;
use rust_socketio::asynchronous::ClientBuilder;
use rust_socketio::Payload;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

use events::websocket::ListenToResult;
use events::{AppChangedEvent, TicketUpdatedEvent};

pub trait Namespace {
    type Message: for<'de> serde::Deserialize<'de> + Send + Sync + 'static;

    fn namespace() -> &'static str;

    fn callback_event() -> &'static str;
}

pub struct TicketNamespace;

impl Namespace for TicketNamespace {
    type Message = (Uuid, TicketUpdatedEvent);

    fn namespace() -> &'static str {
        events::TICKETS_NAMESPACE
    }

    fn callback_event() -> &'static str {
        events::event_channels::TICKET_UPDATED_EVENT
    }
}

pub struct AppChangesNamespace;

impl Namespace for AppChangesNamespace {
    type Message = (Uuid, AppChangedEvent);

    fn namespace() -> &'static str {
        events::APP_CHANGES_NAMESPACE
    }

    fn callback_event() -> &'static str {
        events::event_channels::APP_CHANGED_EVENT
    }
}

pub struct TicketSocketConfig {
    pub server_url: String,
    pub token: String,
}

pub async fn connect<N: Namespace>(
    config: &TicketSocketConfig,
) -> TicketsResult<(Client, UnboundedReceiver<N::Message>)> {
    println!("Connecting to the server...");
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    let client = ClientBuilder::new(&config.server_url)
        .namespace(N::namespace())
        .on(N::callback_event(), move |payload, client| {
            let sender = sender.clone();
            Box::pin(async move {
                match payload {
                    Payload::Binary(_) => {
                        log::error!("Received binary payload.");
                    }
                    Payload::String(value) => {
                        let event: N::Message = match serde_json::from_str(&value) {
                            Ok(event) => event,
                            Err(err) => {
                                log::error!("Could not deserialize event {}", err);
                                return;
                            }
                        };
                        if sender.send(event).is_err() {
                            let _ = client.disconnect().await;
                        }
                    }
                }
            })
        })
        .connect()
        .await?;

    Ok((client, receiver))
}

#[allow(async_fn_in_trait)]
pub trait TicketsWebsocketClientExt {
    async fn listen_to(
        &self,
        app_id: Uuid,
        authorized_app_token: Option<String>,
    ) -> TicketsResult<()>;
}

impl TicketsWebsocketClientExt for Client {
    async fn listen_to(
        &self,
        app_id: Uuid,
        authorized_app_token: Option<String>,
    ) -> TicketsResult<()> {
        self.emit_with_ack(
            events::websocket::LISTEN_TO_EVENT_NAME,
            Payload::String(serde_json::to_string(&events::websocket::ListenTo {
                app_id,
                authorized_app_token,
            })?),
            Duration::from_secs(10),
            move |payload, _client| {
                Box::pin(async move {
                    match payload {
                        Payload::Binary(_) => {
                            log::error!("Received binary payload.");
                        }
                        Payload::String(value) => {
                            let result: ListenToResult = match serde_json::from_str(&value) {
                                Ok(result) => result,
                                Err(err) => {
                                    log::error!("Could not deserialize listen to result {}", err);
                                    return;
                                }
                            };

                            match result {
                                ListenToResult::Success => {
                                    log::info!("Listening to {}.", app_id);
                                }
                                ListenToResult::Failure => {
                                    log::info!("Could not listen to {}.", app_id);
                                }
                            }
                        }
                    }
                })
            },
        )
        .await?;
        Ok(())
    }
}
