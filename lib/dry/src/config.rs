use errors::TicketsResult;
use serde::de::DeserializeOwned;

pub fn load_config<T: DeserializeOwned>() -> TicketsResult<T> {
    let config = std::fs::read_to_string("config.json")?;
    Ok(serde_json::from_str(&config)?)
}
