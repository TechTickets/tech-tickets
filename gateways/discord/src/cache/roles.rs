use std::sync::Arc;

use auth::UserRole;
use bimap::BiMap;
use serenity::all::{Color, Context, GuildId, RoleId};
use tokio::sync::RwLock;

use errors::TicketsResult;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum RolePurpose {
    Staff,
    Management,
}

impl From<UserRole> for RolePurpose {
    fn from(value: UserRole) -> Self {
        match value {
            UserRole::Staff => RolePurpose::Staff,
            UserRole::Management => RolePurpose::Management,
        }
    }
}

impl From<RolePurpose> for UserRole {
    fn from(value: RolePurpose) -> Self {
        match value {
            RolePurpose::Staff => UserRole::Staff,
            RolePurpose::Management => UserRole::Management,
        }
    }
}

const STAFF_ROLE_NAME: &str = "Tech Tickets Staff";
const MANAGEMENT_ROLE_NAME: &str = "Tech Tickets Management";

impl RolePurpose {
    pub fn role_name(self) -> &'static str {
        match self {
            RolePurpose::Staff => STAFF_ROLE_NAME,
            RolePurpose::Management => MANAGEMENT_ROLE_NAME,
        }
    }

    pub fn role_color(self) -> Color {
        match self {
            RolePurpose::Staff => Color::GOLD,
            RolePurpose::Management => Color::RED,
        }
    }
}

pub struct RolesCache {
    inner: Arc<RwLock<BiMap<(RolePurpose, GuildId), RoleId>>>,
}

impl Default for RolesCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BiMap::new())),
        }
    }
}

impl RolesCache {
    pub async fn init_roles_cache(
        &self,
        ctx: &Context,
        guilds: impl Iterator<Item = GuildId>,
    ) -> TicketsResult<()> {
        let mut inner = self.inner.write().await;

        let size_hint = guilds.size_hint().0 * std::mem::variant_count::<RolePurpose>();

        let existing_capacity = inner.capacity() - inner.len();
        if size_hint > existing_capacity {
            inner.reserve(size_hint - existing_capacity);
        }

        for guild_id in guilds {
            for role in guild_id.roles(&ctx.http).await? {
                match role.1.name.as_str() {
                    STAFF_ROLE_NAME => {
                        inner.insert((RolePurpose::Staff, guild_id), role.0);
                    }
                    MANAGEMENT_ROLE_NAME => {
                        inner.insert((RolePurpose::Management, guild_id), role.0);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub async fn insert(&self, guild_id: GuildId, purpose: RolePurpose, role_id: RoleId) {
        let mut inner = self.inner.write().await;
        inner.insert((purpose, guild_id), role_id);
    }

    pub async fn get_role_id(&self, guild_id: GuildId, purpose: RolePurpose) -> Option<RoleId> {
        let inner = self.inner.read().await;
        inner.get_by_left(&(purpose, guild_id)).copied()
    }
}
