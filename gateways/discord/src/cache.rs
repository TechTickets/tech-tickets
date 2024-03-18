pub mod roles;
pub mod users;

use std::collections::HashMap;
use std::sync::Arc;

use bimap::BiMap;
use tokio::sync::RwLock;
use uuid::Uuid;

struct GenericIdCacheInner<Id, Purpose> {
    id_to_purpose_map: HashMap<Id, BiMap<Purpose, Uuid>>,
    app_purpose_to_id: HashMap<(Uuid, Purpose), Id>,
}

impl<Id, Purpose> Default for GenericIdCacheInner<Id, Purpose> {
    fn default() -> Self {
        Self {
            id_to_purpose_map: HashMap::new(),
            app_purpose_to_id: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct GenericIdCache<Id, Purpose> {
    inner: Arc<RwLock<GenericIdCacheInner<Id, Purpose>>>,
}

impl<Id, Purpose> Default for GenericIdCache<Id, Purpose> {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Default::default())),
        }
    }
}

impl<Id: 'static, Purpose: 'static> GenericIdCache<Id, Purpose>
where
    Id: Eq + std::hash::Hash + Copy,
    Purpose: Eq + std::hash::Hash + Copy,
{
    pub async fn populate<'a, I: Iterator<Item = &'a (Id, Purpose, Uuid)>>(&self, iter: I) {
        let mut inner = self.inner.write().await;

        let size_hint = iter.size_hint().0;

        let existing_capacity = inner.id_to_purpose_map.capacity() - inner.id_to_purpose_map.len();
        if size_hint > existing_capacity {
            inner
                .id_to_purpose_map
                .reserve(size_hint - existing_capacity);
        }

        let existing_capacity = inner.app_purpose_to_id.capacity() - inner.app_purpose_to_id.len();
        if size_hint > existing_capacity {
            inner
                .app_purpose_to_id
                .reserve(size_hint - existing_capacity);
        }

        for (id, purpose, app_id) in iter {
            match inner.id_to_purpose_map.get_mut(id) {
                None => {
                    let mut new_map = BiMap::new();
                    new_map.insert(*purpose, *app_id);
                    inner.id_to_purpose_map.insert(*id, new_map);
                }
                Some(map) => {
                    map.insert(*purpose, *app_id);
                }
            }
            inner.app_purpose_to_id.insert((*app_id, *purpose), *id);
        }
    }

    pub async fn get_id(&self, app_id: Uuid, purpose: Purpose) -> Option<Id> {
        let inner = self.inner.read().await;
        inner.app_purpose_to_id.get(&(app_id, purpose)).cloned()
    }

    pub async fn get_app_id(&self, id: Id, purpose: Purpose) -> Option<Uuid> {
        let inner = self.inner.read().await;
        inner
            .id_to_purpose_map
            .get(&id)
            .and_then(|map| map.get_by_left(&purpose))
            .cloned()
    }
}
