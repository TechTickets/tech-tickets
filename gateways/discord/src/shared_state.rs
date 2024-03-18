use crate::channels::ChannelCache;
use crate::guilds::GuildCache;
use errors::{MiscError, TicketsResult};
use serenity::all::Http;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct SharedAppState {
    http: Arc<RwLock<Option<Arc<Http>>>>,
    pub guild_cache: GuildCache,
    pub channel_cache: ChannelCache,
}

impl SharedAppState {
    pub async fn set_http(&self, new_http: Arc<Http>) {
        let mut write = self.http.write().await;
        *write = Some(new_http);
    }

    pub async fn _http(&self) -> Option<Arc<Http>> {
        self.http.read().await.clone()
    }

    pub async fn _require_http(&self) -> TicketsResult<Arc<Http>> {
        self._http()
            .await
            .ok_or(MiscError::MissingHttpClient.into())
    }
}
