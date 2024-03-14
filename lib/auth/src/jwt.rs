use crate::UserRole;
use errors::{AuthorizationError, TicketsError, TicketsResult};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum JwtAccessor {
    DiscordSystem,
    DiscordStaffMember {
        user_id: u64,
        authorized_apps: HashSet<Uuid>,
        role: UserRole,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct JwtData {
    pub accessor: JwtAccessor,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct JwtClaim {
    data: JwtData,
    pub exp: i64,
}

#[derive(serde::Deserialize)]
pub struct JwtKeyPathsConfig {
    pub public_key_location: String,
    pub private_key_location: String,
}

pub struct JwtConfig {
    public_key: String,
    private_key: String,
}

impl JwtConfig {
    pub fn from_key_paths<S1: Into<String>, S2: Into<String>>(
        public_key: S1,
        private_key: S2,
    ) -> TicketsResult<Self> {
        Ok(Self {
            public_key: std::fs::read_to_string(public_key.into())?,
            private_key: std::fs::read_to_string(private_key.into())?,
        })
    }

    pub fn from_keys<S1: Into<String>, S2: Into<String>>(public_key: S1, private_key: S2) -> Self {
        Self {
            public_key: public_key.into(),
            private_key: private_key.into(),
        }
    }

    pub fn generate(
        &self,
        jwt_data: JwtData,
        ttl: core::time::Duration,
    ) -> TicketsResult<(String, JwtClaim)> {
        generate_jwt_token(jwt_data, ttl, &self.private_key)
    }

    pub fn verify(&self, token: &str) -> TicketsResult<JwtData> {
        Ok(verify_jwt_token(&self.public_key, token)?)
    }
}

impl TryFrom<JwtKeyPathsConfig> for JwtConfig {
    type Error = TicketsError;

    fn try_from(value: JwtKeyPathsConfig) -> Result<Self, Self::Error> {
        Self::from_key_paths(value.public_key_location, value.private_key_location)
    }
}

pub fn generate_jwt_token(
    jwt_data: JwtData,
    ttl: core::time::Duration,
    private_key: &str,
) -> TicketsResult<(String, JwtClaim)> {
    let exp = (chrono::Utc::now()
        + chrono::Duration::from_std(ttl).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid duration")
        })?)
    .timestamp();
    let claims = JwtClaim {
        data: jwt_data,
        exp,
    };

    let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key.as_bytes())
        .map_err(AuthorizationError::from)?;
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &encoding_key,
    )
    .map_err(AuthorizationError::from)?;

    Ok((token, claims))
}

pub fn verify_jwt_token(public_key: &String, token: &str) -> Result<JwtData, AuthorizationError> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);

    let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(public_key.as_bytes())?;
    let decoded = jsonwebtoken::decode::<JwtClaim>(token, &decoding_key, &validation)?;

    Ok(decoded.claims.data)
}
