use std::sync::Arc;

use auth::jwt::{JwtAccessor, JwtConfig, JwtData};
use errors::{TicketsError, TicketsResult};
use socketioxide::extract::{AckSender, Data, SocketRef, State};
use socketioxide::layer::SocketIoLayer;
use socketioxide::SocketIo;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use uuid::Uuid;

use events::event_channels::{APP_CHANGED_EVENT, TICKET_SUBMITTED_EVENT};
use events::websocket::SocketAuthData;
use events::websocket::{ListenTo, ListenToResult};
use events::{PublishedMessage, TicketEvent, APP_CHANGES_NAMESPACE, TICKETS_NAMESPACE};

#[derive(Clone)]
struct SocketIoState {
    jwt: Arc<JwtConfig>,
}

macro_rules! auth {
    ($caller:ident = ($socket:ident, $state:ident, $data:ident).into()) => {
        let $caller = match $state.jwt.verify(&$data.token) {
            Ok($caller) => $caller,
            Err(_) => {
                let _ = $socket.disconnect();
                return;
            }
        };
    };
}

async fn listen_handler(
    socket: SocketRef,
    State(state): State<SocketIoState>,
    Data(data): Data<ListenTo>,
    ack: AckSender,
) {
    let caller = socket.extensions.get::<JwtData>();
    let JwtData { accessor } = match caller.map(|caller| caller.value().clone()) {
        Some(caller) => caller,
        None => {
            let _ = socket.disconnect();
            return;
        }
    };

    match accessor {
        JwtAccessor::DiscordSystem => {
            let _ = socket.join(data.app_id.to_string());
            if ack.send(ListenToResult::Success).is_err() {
                let _ = socket.disconnect();
            }
        }
        JwtAccessor::DiscordStaffMember {
            user_id,
            authorized_apps,
            ..
        } => {
            let app_listen_authorized = if authorized_apps.contains(&data.app_id) {
                true
            } else {
                let token = {
                    match data.authorized_app_token {
                        None => {
                            if ack.send(ListenToResult::Failure).is_err() {
                                let _ = socket.disconnect();
                            }
                            return;
                        }
                        Some(token) => token,
                    }
                };

                let jwt_data = match state.jwt.verify(&token) {
                    Ok(jwt_data) => jwt_data,
                    Err(_) => {
                        if ack.send(ListenToResult::Failure).is_err() {
                            let _ = socket.disconnect();
                        }
                        return;
                    }
                };
                match jwt_data.accessor {
                    JwtAccessor::DiscordStaffMember {
                        user_id: user_id_authed,
                        authorized_apps: authed_apps,
                        ..
                    } if user_id_authed == user_id => authed_apps.contains(&data.app_id),
                    // guarantor
                    JwtAccessor::DiscordSystem => true,
                    _ => false,
                }
            };

            if app_listen_authorized {
                let _ = socket.join(data.app_id.to_string());
                if ack.send(ListenToResult::Success).is_err() {
                    let _ = socket.disconnect();
                }
            } else if ack.send(ListenToResult::Failure).is_err() {
                let _ = socket.disconnect();
            }
        }
    }
}

async fn authed_continue(
    socket: SocketRef,
    caller: JwtData,
) -> Result<(), (SocketRef, TicketsError)> {
    socket.extensions.insert(caller);
    socket.on(events::websocket::LISTEN_TO_EVENT_NAME, listen_handler);
    Ok(())
}

async fn prepare_auth(
    socket: SocketRef,
    State(state): State<SocketIoState>,
    Data(data): Data<SocketAuthData>,
) {
    auth!(caller = (socket, state, data).into());

    if let Err((socket, _)) = authed_continue(socket, caller).await {
        let _ = socket.disconnect();
    }
}

pub fn setup_server(
    jwt_config: Arc<JwtConfig>,
    mut recv_handle: UnboundedReceiver<PublishedMessage>,
) -> (JoinHandle<TicketsResult<()>>, SocketIoLayer) {
    let (layer, io) = SocketIo::builder()
        .with_state(SocketIoState { jwt: jwt_config })
        .build_layer();

    io.ns(APP_CHANGES_NAMESPACE, prepare_auth);
    io.ns(TICKETS_NAMESPACE, prepare_auth);

    log::info!("Websocket I/O Prepared");

    let broadcast = |io: &SocketIo, namespace: &str, app_id: Uuid| {
        io.of(namespace)
            .map(|ops| ops.to(app_id.to_string()).broadcast())
    };

    let message_receiver_handle: JoinHandle<TicketsResult<()>> = tokio::spawn(async move {
        while let Some(msg) = recv_handle.recv().await {
            match msg.event {
                TicketEvent::AppChanged(app_changed) => {
                    if let Some(broadcast) = broadcast(&io, APP_CHANGES_NAMESPACE, msg.app_id) {
                        broadcast.emit(APP_CHANGED_EVENT, (msg.app_id, app_changed))?;
                    }
                }
                TicketEvent::TicketSubmitted(ticket_submitted) => {
                    if let Some(broadcast) = broadcast(&io, TICKETS_NAMESPACE, msg.app_id) {
                        broadcast.emit(TICKET_SUBMITTED_EVENT, (msg.app_id, ticket_submitted))?;
                    }
                }
            }
        }

        Ok(())
    });

    (message_receiver_handle, layer)
}
