use std::time::Duration;

use crate::state::DiscordAppState;
use moka::policy::EvictionPolicy;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use serenity::all::UserId;
use tickets_common::errors::SdkResult;
use tickets_common::jwt::JwtAccessor;
use tickets_common::requests::sdk::{InternalSdk, SignedTicketClient};
use tickets_common::requests::SdkExecutor;

#[derive(Clone)]
pub struct User {
    pub client: SignedTicketClient,
}

impl SdkExecutor for User {
    async fn call<T: for<'de> Deserialize<'de>, S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        query_params: Q,
    ) -> SdkResult<T> {
        self.client.call(method, path, query_params).await
    }

    async fn call_with_body<
        T: for<'de> Deserialize<'de>,
        B: Serialize,
        S: Into<String>,
        Q: Serialize,
    >(
        &self,
        method: Method,
        path: S,
        body: B,
        query_params: Q,
    ) -> SdkResult<T> {
        self.client
            .call_with_body(method, path, body, query_params)
            .await
    }

    async fn invoke<S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        query_params: Q,
    ) -> SdkResult<StatusCode> {
        self.client.invoke(method, path, query_params).await
    }

    async fn invoke_with_body<B: Serialize, S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        body: B,
        query_params: Q,
    ) -> SdkResult<StatusCode> {
        self.client
            .invoke_with_body(method, path, body, query_params)
            .await
    }
}

impl User {
    pub fn staff(app_state: &DiscordAppState, user_id: UserId) -> Self {
        Self {
            client: app_state
                .internal_sdk
                .sign_client(
                    JwtAccessor::DiscordStaffMember {
                        user_id: user_id.get(),
                    },
                    InternalSdk::DEFAULT_TTL,
                )
                .expect("Failed to sign new sdk client"),
        }
    }
}

pub struct UsersCache {
    inner: moka::future::Cache<UserId, User>,
}

impl Default for UsersCache {
    fn default() -> Self {
        let inner = moka::future::CacheBuilder::new(500)
            .name("UsersCache")
            .eviction_policy(EvictionPolicy::tiny_lfu())
            .time_to_idle(Duration::from_secs(5 * 60))
            .build();
        Self { inner }
    }
}

impl UsersCache {
    pub async fn get_or_insert(&self, user_id: UserId, func: impl FnOnce() -> User) -> User {
        self.inner.get_with(user_id, async move { func() }).await
    }
}
