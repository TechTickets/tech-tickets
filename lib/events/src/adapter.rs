#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub enum Adapter {
    #[cfg(feature = "redis")]
    #[serde(rename = "redis")]
    Redis,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AdapterConfig {
    #[serde(rename = "type")]
    pub adapter_type: Adapter,

    #[cfg(feature = "redis")]
    pub redis: Option<RedisConfig>,
}

#[cfg(feature = "redis")]
#[derive(serde::Deserialize, Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}
