pub mod jwt;

use errors::ParsingError;
#[cfg(feature = "axum")]
pub use server_handle::AuthedCaller;
use std::fmt::Display;

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy, Ord, PartialOrd,
)]
pub enum UserRole {
    Staff,
    Management,
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Staff => write!(f, "staff"),
            UserRole::Management => write!(f, "management"),
        }
    }
}

impl TryFrom<String> for UserRole {
    type Error = ParsingError;

    fn try_from(role_name: String) -> Result<Self, Self::Error> {
        Ok(match role_name.as_str() {
            "staff" => UserRole::Staff,
            "management" => UserRole::Management,
            _ => return Err(ParsingError::InvalidRole(role_name)),
        })
    }
}

#[cfg(feature = "axum")]
mod server_handle {
    use crate::UserRole;
    use std::collections::HashSet;
    use std::sync::Arc;
    use uuid::Uuid;

    use super::jwt::{JwtAccessor, JwtConfig, JwtData};
    use errors::{AuthorizationError, TicketsError, TicketsResult};

    pub enum ChannelType {
        Discord,
    }

    pub struct AuthedChannel {
        pub channel_type: ChannelType,
    }

    pub struct AuthedUser {
        pub user_id: u64,
        pub pre_authorized_apps: HashSet<Uuid>,
        pub role: UserRole,
    }

    pub enum AuthedCaller {
        User(AuthedUser),
        Channel(AuthedChannel),
    }

    impl AuthedCaller {
        pub fn require_user(self) -> TicketsResult<AuthedUser> {
            match self {
                AuthedCaller::User(user) => Ok(user),
                AuthedCaller::Channel(_) => Err(AuthorizationError::ChannelCannotAccessResource)?,
            }
        }

        pub fn require_channel(self) -> TicketsResult<AuthedChannel> {
            match self {
                AuthedCaller::User(_) => Err(AuthorizationError::UserCannotAccessResource)?,
                AuthedCaller::Channel(channel) => Ok(channel),
            }
        }
    }

    impl From<JwtData> for AuthedCaller {
        fn from(value: JwtData) -> Self {
            match value.accessor {
                JwtAccessor::DiscordSystem => AuthedCaller::Channel(AuthedChannel {
                    channel_type: ChannelType::Discord,
                }),
                JwtAccessor::DiscordStaffMember {
                    user_id,
                    authorized_apps,
                    role,
                } => AuthedCaller::User(AuthedUser {
                    user_id,
                    pre_authorized_apps: authorized_apps,
                    role,
                }),
            }
        }
    }

    fn get_bearer_token(header: &str) -> Option<String> {
        let prefix_len = "Bearer ".len();

        match header.len() {
            l if l < prefix_len => None,
            _ => Some(header[prefix_len..].to_string()),
        }
    }

    #[axum::async_trait]
    impl<S> axum::extract::FromRequestParts<S> for AuthedCaller
    where
        Arc<JwtConfig>: axum::extract::FromRef<S>,
        S: Send + Sync,
    {
        type Rejection = TicketsError;

        async fn from_request_parts(
            parts: &mut axum::http::request::Parts,
            state: &S,
        ) -> Result<Self, Self::Rejection> {
            let jwt_config: Arc<JwtConfig> = axum::extract::FromRef::<S>::from_ref(state);

            let auth_header = parts
                .headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .ok_or(AuthorizationError::MissingBearerToken)?;

            let bearer =
                get_bearer_token(auth_header).ok_or(AuthorizationError::MalformedBearerToken)?;

            Ok(jwt_config.verify(&bearer)?.into())
        }
    }
}
