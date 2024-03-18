use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use uuid::Uuid;

use errors::TicketsResult;
use events::TicketUpdatedEvent;

use crate::shared_state::SharedAppState;

struct TicketEventsState {
    _shared_state: SharedAppState,
}

pub fn read_ticket_events(
    mut receiver: UnboundedReceiver<(Uuid, TicketUpdatedEvent)>,
    shared_app_state: SharedAppState,
) -> JoinHandle<TicketsResult<()>> {
    tokio::spawn(async move {
        let _state = TicketEventsState {
            _shared_state: shared_app_state,
        };

        while let Some((app_id, event)) = receiver.recv().await {
            log::info!("Received ticket update event for app {app_id}: {event:?}");
        }
        Ok(())
    })
}
