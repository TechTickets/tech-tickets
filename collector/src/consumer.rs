use axum::Router;
use sdk::routes::consumer::*;

use crate::axum_ext::ApplySdkRoute;
use crate::GlobalState;

pub fn extend_router(router: Router<GlobalState>) -> Router<GlobalState> {
    router.merge(Router::new().sdk_route::<SubmitTicket>(submit_ticket::route_handler))
}

pub mod submit_ticket {
    use axum::extract::State;
    use axum::http::HeaderMap;
    use axum::Json;
    use errors::{AuthorizationError, TicketsResult};
    use events::TicketSubmittedEvent;
    use sdk::routes::consumer::{SubmitTicketBody, SubmitTicketResponse};
    use uuid::Uuid;

    use crate::axum_ext::RequireHeaderFromHeaderMap;
    use crate::GlobalState;

    pub(super) async fn route_handler(
        State(state): State<GlobalState>,
        headers: HeaderMap,
        Json(body): Json<SubmitTicketBody>,
    ) -> TicketsResult<Json<SubmitTicketResponse>> {
        let gateway = headers.require_header("x-gateway")?;
        let app_id = body.app_id;

        let pg_client = &state.pg_client;

        // determine if the app had this gateway enabled
        let enabled = sqlx::query!(
            "SELECT * FROM gateway WHERE app_id = $1 AND name = $2",
            &app_id,
            &gateway
        )
        .fetch_optional(pg_client)
        .await?
        .is_some();

        if !enabled {
            return Err(AuthorizationError::GatewayNotEnabled { gateway })?;
        }

        let ticket_id = Uuid::new_v4();

        // insert the ticket
        sqlx::query!(
            "INSERT INTO ticket (id, app_id, message, gateway) VALUES ($1, $2, $3, $4)",
            &ticket_id,
            &app_id,
            &body.message,
            &gateway
        )
        .execute(pg_client)
        .await?;

        state.emitter.publish_tickets_event(
            app_id,
            TicketSubmittedEvent {
                message: body.message,
            }
            .into(),
        )?;

        Ok(Json(SubmitTicketResponse { ticket_id }))
    }
}
