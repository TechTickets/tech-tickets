use auth::jwt::JwtConfig;
use auth::UserRole;
use errors::{AuthorizationError, TicketsResult};
use socketio_emitter::adapter::TicketsEventEmitter;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct GlobalState {
    pub pg_client: Pool<Postgres>,
    pub jwt_config: Arc<JwtConfig>,
    pub emitter: Arc<dyn TicketsEventEmitter + Send + Sync>,
}

impl GlobalState {
    pub async fn validate_user_role(
        &self,
        user_id: u64,
        required_role: UserRole,
        app_id: Uuid,
    ) -> TicketsResult<UserRole> {
        let user_role: UserRole = sqlx::query!(
            "SELECT role FROM user_app WHERE app_id = $1 AND user_id = $2",
            &app_id,
            user_id as i64
        )
        .fetch_optional(&self.pg_client)
        .await?
        .map(|record| serde_json::from_str(&record.role))
        .ok_or(AuthorizationError::CannotAccessApp)??;

        if user_role < required_role {
            return Err(AuthorizationError::InsufficientRole)?;
        }

        Ok(user_role)
    }
}
