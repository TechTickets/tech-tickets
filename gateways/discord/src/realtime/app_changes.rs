use crate::shared_state::SharedAppState;
use errors::TicketsResult;
use events::AppChangedEvent;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use uuid::Uuid;

struct AppChangesState {
    _shared_state: SharedAppState,
}

pub fn read_app_changes(
    mut receiver: UnboundedReceiver<(Uuid, AppChangedEvent)>,
    shared_app_state: SharedAppState,
) -> JoinHandle<TicketsResult<()>> {
    tokio::spawn(async move {
        let _state = AppChangesState {
            _shared_state: shared_app_state,
        };

        while let Some((app_id, event)) = receiver.recv().await {
            log::info!("Received app change event for app {app_id}: {event:?}");
        }
        Ok(())
    })
}
