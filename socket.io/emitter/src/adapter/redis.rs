use events::PublishedMessage;
macro_rules! adapter_impl {
    ($($t:ty),*) => {
        $(
            impl super::TicketsEventEmitter for $t {
                fn publish_tickets_event(
                    &self,
                    app_id: uuid::Uuid,
                    event: events::TicketEvent,
                ) -> errors::TicketsResult<()> {
                    let mut con = self.get_connection()?;
                    redis::Commands::publish(
                        &mut con,
                        events::TICKETS_LIVE_EVENTS_CHANNEL,
                        serde_json::to_string(&PublishedMessage {
                            app_id,
                            event,
                        }).unwrap()
                    ).map_err(Into::into)
                }
            }
        )*
    };
}

adapter_impl!(
    redis::Client,
    std::rc::Rc<redis::Client>,
    std::sync::Arc<redis::Client>
);
