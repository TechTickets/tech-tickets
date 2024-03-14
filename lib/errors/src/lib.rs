pub type TicketsResult<T> = Result<T, TicketsError>;

#[derive(thiserror::Error, Debug)]
pub enum MiscError {
    #[error("Could not find guild data.")]
    GuildDataNotFound,
}

impl MiscError {
    #[cfg(feature = "axum")]
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            MiscError::GuildDataNotFound => axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParsingError {
    #[error("Failed to parse Guild Purpose, `{0}` is not valid.")]
    InvalidGuildPurpose(String),
    #[cfg(feature = "url")]
    #[error("Failed to parse URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("Missing required header: {header}")]
    MissingRequiredHeader { header: String },
}

#[derive(thiserror::Error, Debug)]
pub enum AuthorizationError {
    #[error("JWT Error: {0}")]
    JsonWebToken(#[from] jsonwebtoken::errors::Error),
    #[error("Users cannot access this resource.")]
    UserCannotAccessResource,
    #[error("Channels cannot access this resource.")]
    ChannelCannotAccessResource,
    #[error("Missing bearer token in header.")]
    MissingBearerToken,
    #[error("Malformed bearer token in header.")]
    MalformedBearerToken,
    #[error("The gateway `{gateway}` is not enabled for this app.")]
    GatewayNotEnabled { gateway: String },
    #[error("You do not have permission to modify this app.")]
    CannotAccessApp,
    #[error("Your role does not have permission to access this resource.")]
    InsufficientRole,
}

impl AuthorizationError {
    #[cfg(feature = "axum")]
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            AuthorizationError::MissingBearerToken | AuthorizationError::MalformedBearerToken => {
                axum::http::StatusCode::BAD_REQUEST
            }
            AuthorizationError::JsonWebToken(_) => axum::http::StatusCode::UNAUTHORIZED,
            _ => axum::http::StatusCode::FORBIDDEN,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct NetworkError {
    pub reason: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TicketsError {
    #[error(transparent)]
    Authorization(#[from] AuthorizationError),
    #[error(transparent)]
    Parsing(#[from] ParsingError),
    #[cfg(feature = "sqlx")]
    #[error("SQLx Error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[cfg(feature = "sqlx")]
    #[error("Error migrating sql: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("Error Parsing JSON: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[cfg(feature = "redis")]
    #[error("Redis Error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),
    #[cfg(feature = "reqwest")]
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "socketioxide")]
    #[error("WebSocket Broadcast Error: {0}")]
    WebsocketBroadcastError(#[from] socketioxide::BroadcastError),
    #[cfg(feature = "rust_socketio")]
    #[error("WebSocket Client Error: {0}")]
    WebsocketClientError(#[from] rust_socketio::Error),
    #[error(transparent)]
    Misc(#[from] MiscError),
    #[error("Network Error: {}", .0.reason)]
    Network(NetworkError),
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for TicketsError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let status = match &self {
            TicketsError::Parsing(_) => axum::http::StatusCode::BAD_REQUEST,
            TicketsError::Authorization(err) => err.status_code(),
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            axum::Json(NetworkError {
                reason: self.to_string(),
            }),
        )
            .into_response()
    }
}
