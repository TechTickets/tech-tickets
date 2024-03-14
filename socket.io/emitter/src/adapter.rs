use errors::TicketsResult;

#[cfg(feature = "redis")]
pub mod redis;

pub trait TicketsEventEmitter {
    fn publish_tickets_event(
        &self,
        app_id: uuid::Uuid,
        event: events::TicketEvent,
    ) -> TicketsResult<()>;
}
