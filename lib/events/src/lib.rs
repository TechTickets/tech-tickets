#![feature(concat_idents)]
pub mod adapter;

use auth::UserRole;
use uuid::Uuid;

pub const APP_CHANGES_NAMESPACE: &str = "/app_changes";
pub const TICKETS_NAMESPACE: &str = "/tickets";
pub const TICKETS_LIVE_EVENTS_CHANNEL: &str = "tickets_live_events";

macro_rules! event_hierarchy {
    (
        $glob:ident {
            $(
                $parent:ident($parent_event:ident) {
                    $(
                        $child:ident($child_event:ident) {
                            $(
                                $field:ident: $type:ty,
                            )*
                        }
                    ),*
                }
            ),*
        }
    ) => {
        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        pub enum $glob {
            $(
                $parent($parent_event),
            )*
        }

        $(
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            pub enum $parent_event {
                $(
                $child($child_event),
                )*
            }

            impl From<$parent_event> for $glob {
                fn from(event: $parent_event) -> Self {
                    $glob::$parent(event)
                }
            }

            $(
                #[derive(serde::Serialize, serde::Deserialize, Debug)]
                pub struct $child_event {
                    pub $($field: $type),*
                }

                impl From<$child_event> for $parent_event {
                    fn from(event: $child_event) -> Self {
                        $parent_event::$child(event)
                    }
                }

                impl From<$child_event> for $glob {
                    fn from(event: $child_event) -> Self {
                        $glob::$parent($parent_event::$child(event))
                    }
                }
            )*
        )*
    };
}

event_hierarchy! {
    TicketEvent {
        AppChanged(AppChangedEvent) {
            StaffPromoted(StaffPromotedEvent) {
                user_id: u64,
                role: UserRole,
            }
        },
        TicketUpdated(TicketUpdatedEvent) {
            TicketSubmitted(TicketSubmittedEvent) {
                message: String,
            }
        }
    }
}

pub mod event_channels {
    pub const APP_CHANGED_EVENT: &str = "app_changed";

    pub const TICKET_UPDATED_EVENT: &str = "ticket_updated";
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PublishedMessage {
    pub app_id: Uuid,
    pub event: TicketEvent,
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
