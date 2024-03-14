pub mod adapter;

use auth::UserRole;
use uuid::Uuid;

pub const APP_CHANGES_NAMESPACE: &str = "/app_changes";
pub const TICKETS_NAMESPACE: &str = "/tickets";
pub const TICKETS_LIVE_EVENTS_CHANNEL: &str = "tickets_live_events";

pub mod event_channels {
    pub const APP_CHANGED_EVENT: &str = "app_changed";

    pub const TICKET_SUBMITTED_EVENT: &str = "ticket_submitted";
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PublishedMessage {
    pub app_id: Uuid,
    pub event: TicketEvent,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum TicketEvent {
    AppChanged(AppChangedEvent),
    TicketSubmitted(TicketSubmittedEvent),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TicketSubmittedEvent {
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AppChangedEvent {
    StaffPromoted { staff_id: i64, role: UserRole },
}

impl From<TicketEvent> for TicketSubmittedEvent {
    fn from(event: TicketEvent) -> Self {
        match event {
            TicketEvent::TicketSubmitted(submitted) => submitted,
            _ => panic!("Tried to convert non-TicketSubmittedEvent to TicketSubmittedEvent"),
        }
    }
}

impl From<TicketEvent> for AppChangedEvent {
    fn from(event: TicketEvent) -> Self {
        match event {
            TicketEvent::AppChanged(app_changed) => app_changed,
            _ => panic!("Tried to convert non-AppChangedEvent to AppChangedEvent"),
        }
    }
}

pub mod websocket {
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct SocketAuthData {
        pub token: String,
    }

    use uuid::Uuid;

    pub const LISTEN_TO_EVENT_NAME: &str = "listen_to";

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct ListenTo {
        pub app_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub authorized_app_token: Option<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub enum ListenToResult {
        Success,
        Failure,
    }
}
