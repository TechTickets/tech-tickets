use std::collections::HashMap;
use std::sync::Arc;

use serenity::all::GuildId;
use tickets_common::requests::staff::GuildPurpose;
use tokio::sync::RwLock;
use uuid::Uuid;

struct GuildCacheInner {
    app_purpose_map: HashMap<(Uuid, GuildPurpose), GuildId>,
    guild_app_map: HashMap<GuildId, Uuid>,
}

pub struct GuildCache {
    inner: Arc<RwLock<GuildCacheInner>>,
}

impl Default for GuildCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(GuildCacheInner {
                app_purpose_map: HashMap::new(),
                guild_app_map: HashMap::new(),
            })),
        }
    }
}

impl GuildCache {
    pub async fn populate(&self, entries: impl Iterator<Item = (GuildId, GuildPurpose, Uuid)>) {
        let mut inner = self.inner.write().await;

        let size_hint = entries.size_hint().0;

        let existing_capacity = inner.app_purpose_map.capacity() - inner.app_purpose_map.len();
        if size_hint > existing_capacity {
            inner.app_purpose_map.reserve(size_hint - existing_capacity);
        }

        let existing_capacity = inner.guild_app_map.capacity() - inner.guild_app_map.len();
        if size_hint > existing_capacity {
            inner.guild_app_map.reserve(size_hint - existing_capacity);
        }

        for (guild_id, purpose, app_id) in entries {
            inner.app_purpose_map.insert((app_id, purpose), guild_id);
            inner.guild_app_map.insert(guild_id, app_id);
        }
    }

    pub async fn insert(&self, guild_id: GuildId, purpose: GuildPurpose, app_id: Uuid) {
        let mut inner = self.inner.write().await;
        inner.app_purpose_map.insert((app_id, purpose), guild_id);
        inner.guild_app_map.insert(guild_id, app_id);
    }

    pub async fn get_app_id(&self, guild_id: GuildId) -> Option<Uuid> {
        let inner = self.inner.read().await;
        inner.guild_app_map.get(&guild_id).copied()
    }
}
