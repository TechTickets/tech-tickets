use axum::Router;

use sdk::routes::staff::{CreateApp, Login, ToggleGateway};

use crate::axum_ext::ApplySdkRoute;
use crate::GlobalState;

pub fn extend_router(router: Router<GlobalState>) -> Router<GlobalState> {
    router.merge(
        Router::new()
            .sdk_route::<CreateApp>(create_app::route_handler)
            .sdk_route::<ToggleGateway>(toggle_gateway::route_handler)
            .sdk_route::<Login>(login::route_handler),
    )
}

pub mod login {
    use axum::extract::State;

    use auth::AuthedCaller;
    use errors::TicketsResult;

    use crate::GlobalState;

    #[axum::debug_handler]
    pub async fn route_handler(
        user: AuthedCaller,
        State(state): State<GlobalState>,
    ) -> TicketsResult<()> {
        let user = user.require_user()?;
        let pg_client = &state.pg_client;

        // write user if they're not in the database already
        sqlx::query!(
            "INSERT INTO tt_user (id) VALUES ($1) ON CONFLICT (id) DO NOTHING",
            user.user_id as i64
        )
        .execute(pg_client)
        .await?;
        Ok(())
    }
}

pub mod toggle_gateway {
    use axum::extract::State;
    use axum::Json;

    use auth::{AuthedCaller, UserRole};
    use errors::TicketsResult;
    use sdk::routes::staff::{ToggleGatewayBody, ToggleGatewayResponse};

    use crate::axum_ext::RequireHeaderFromHeaderMap;
    use crate::GlobalState;

    pub async fn route_handler(
        user: AuthedCaller,
        headers: axum::http::HeaderMap,
        State(state): State<GlobalState>,
        Json(body): Json<ToggleGatewayBody>,
    ) -> TicketsResult<Json<ToggleGatewayResponse>> {
        let user = user.require_user()?;
        let gateway = headers.require_header("x-gateway")?;
        let app_id = body.app_id;

        let pg_client = &state.pg_client;

        state
            .validate_user_role(user.user_id, UserRole::Management, app_id)
            .await?;

        sqlx::query!(
                "INSERT INTO gateway (app_id, name, enabled) VALUES ($1, $2, $3) ON CONFLICT (app_id, name) DO UPDATE SET enabled = $3",
                &app_id, &gateway, &body.enabled,
            )
            .execute(pg_client)
            .await?;

        Ok(Json(ToggleGatewayResponse {
            gateway: gateway.to_string(),
            enabled: body.enabled,
        }))
    }
}

pub mod create_app {
    use axum::extract::State;
    use axum::Json;
    use sqlx::Executor;
    use uuid::Uuid;

    use auth::{AuthedCaller, UserRole};
    use errors::TicketsResult;
    use sdk::routes::staff::{CreateAppBody, CreateAppResponse};

    use crate::GlobalState;

    pub async fn route_handler(
        user: AuthedCaller,
        State(state): State<GlobalState>,
        Json(body): Json<CreateAppBody>,
    ) -> TicketsResult<Json<CreateAppResponse>> {
        let user = user.require_user()?;
        let pg_client = &state.pg_client;

        let mut tx = pg_client.begin().await?;

        let app_id = Uuid::new_v4();

        tx.execute(sqlx::query!(
            "INSERT INTO app (id, name, owner_id) VALUES ($1, $2, $3)",
            &app_id,
            &body.app_name,
            user.user_id as i64
        ))
        .await?;

        tx.execute(sqlx::query!(
            "INSERT INTO user_app (user_id, app_id, role) VALUES ($1, $2, $3)",
            user.user_id as i64,
            &app_id,
            serde_json::to_string(&UserRole::Management)?
        ))
        .await?;

        tx.commit().await?;

        Ok(Json(CreateAppResponse { app_id }))
    }
}

// pub mod link_discord_guild_id {
//     use auth::{AuthedCaller, UserRole};
//     use axum::extract::State;
//     use axum::Json;
//     use errors::TicketsResult;
//     use sdk::routes::staff::LinkDiscordGuildIdBody;
//
//     use crate::GlobalState;
//
//     pub async fn route_handler(
//         user: AuthedCaller,
//         State(state): State<GlobalState>,
//         Json(body): Json<LinkDiscordGuildIdBody>,
//     ) -> TicketsResult<()> {
//         let user = user.require_user()?;
//
//         let app_id = body.app_id;
//         let pg_client = &state.pg_client;
//
//         state
//             .validate_user_role(user.user_id, UserRole::Management, app_id)
//             .await?;
//
//         sqlx::query!(
//             "INSERT INTO discord_guilds (app_id, guild_id, guild_purpose) VALUES ($1, $2, $3) ON CONFLICT (app_id, guild_purpose) DO UPDATE SET guild_id = $2",
//             &app_id, body.guild_id as i64, &body.guild_purpose.to_string()
//         ).execute(pg_client).await?;
//
//         Ok(())
//     }
// }
//
// pub mod get_bulk_discord_app_data {
//     use auth::AuthedCaller;
//     use axum::extract::State;
//     use axum::Json;
//     use errors::TicketsResult;
//     use sdk::routes::staff::{
//         GetBulkGuildDataBody, GetBulkGuildDataResponse, GuildData, GuildPurpose,
//     };
//
//     use crate::GlobalState;
//
//     pub async fn route_handler(
//         user: AuthedCaller,
//         State(state): State<GlobalState>,
//         Json(body): Json<GetBulkGuildDataBody>,
//     ) -> TicketsResult<Json<GetBulkGuildDataResponse>> {
//         user.require_channel()?;
//
//         let guilds = sqlx::query!(
//             "SELECT * FROM discord_guilds WHERE guild_id = ANY($1)",
//             &body
//                 .guild_ids
//                 .iter()
//                 .map(|id| *id as i64)
//                 .collect::<Vec<i64>>(),
//         )
//         .fetch_all(&state.pg_client)
//         .await?
//         .iter()
//         .filter_map(|row| {
//             let guild_id = row.guild_id as u64;
//             let app_id = row.app_id;
//             let guild_purpose: Option<GuildPurpose> = row.guild_purpose.to_string().try_into().ok();
//             guild_purpose.map(|purpose| GuildData {
//                 guild_id,
//                 app_id,
//                 guild_purpose: purpose,
//             })
//         })
//         .collect();
//
//         Ok(Json(GetBulkGuildDataResponse { guild_data: guilds }))
//     }
// }
//
// mod get_guild_data {
//     use auth::AuthedCaller;
//     use axum::extract::{Query, State};
//     use axum::Json;
//     use errors::{MiscError, TicketsResult};
//     use sdk::routes::staff::{GetGuildDataParams, GuildData, GuildPurpose};
//
//     use crate::GlobalState;
//
//     pub async fn route_handler(
//         user: AuthedCaller,
//         State(state): State<GlobalState>,
//         params: Query<GetGuildDataParams>,
//     ) -> TicketsResult<Json<GuildData>> {
//         user.require_channel()?;
//
//         let guild = sqlx::query!(
//             "SELECT * FROM discord_guilds WHERE guild_id = $1",
//             params.guild_id as i64
//         )
//         .fetch_optional(&state.pg_client)
//         .await?;
//
//         let guild = match guild {
//             Some(guild) => guild,
//             None => {
//                 return Err(MiscError::GuildDataNotFound)?;
//             }
//         };
//
//         let guild_id = guild.guild_id as u64;
//         let app_id = guild.app_id;
//         let guild_purpose: GuildPurpose = guild.guild_purpose.to_string().try_into()?;
//
//         Ok(Json(GuildData {
//             guild_id,
//             app_id,
//             guild_purpose,
//         }))
//     }
// }
