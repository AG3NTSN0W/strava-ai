use crate::libs::models::Athlete;
use crate::libs::repository::AthleteRepository;
use crate::libs::strava_client::StravaClient;
use chrono::Utc;
use log::debug;
use moka::future::Cache;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use anyhow::Result;

pub mod controllers;
pub mod libs;

pub use libs::rate_limit::RateLimit;

#[derive(Clone)]
pub struct AppState {
    pub db_pools: Pool<Sqlite>,
    pub client_id: i32,
    pub client_secret: String,
    cache: Arc<Cache<String, String>>,
    pub rate_limit: Arc<RwLock<RateLimit>>,
}

impl AppState {
    pub fn new(client_id: i32, client_secret: String, db_pools: Pool<Sqlite>) -> Arc<Self> {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(3 * 60 * 60)) // 3 hours
            .build();

        Arc::new(Self {
            db_pools: db_pools.clone(),
            client_id,
            client_secret,
            cache: Arc::new(cache),
            rate_limit: Arc::new(RwLock::new(RateLimit::default())),
        })
    }

    /// Get cached access token for an athlete
    async fn get_token_from_cache(&self, athlete_id: i64) -> Option<String> {
        let cache_key = format!("access_token_{athlete_id}");
        self.cache.get(&cache_key).await
    }

    /// Cache access token for an athlete (5-hour TTL)
    pub async fn cache_access_token(&self, athlete_id: i64, token: &str) {
        let cache_key = format!("access_token_{athlete_id}");
        self.cache.insert(cache_key, token.to_string()).await;
    }

    /// Invalidate cached access token for an athlete
    pub async fn invalidate_access_token(&self, athlete_id: i64) {
        let cache_key = format!("access_token_{athlete_id}");
        self.cache.invalidate(&cache_key).await;
    }

    /// Update rate limit from Strava response headers
    pub async fn update_rate_limit(&self, response: &reqwest::Response) {
        let headers = response.headers();
        let limit = headers
            .get("X-RateLimit-Limit")
            .and_then(|v| v.to_str().ok());
        let usage = headers
            .get("X-RateLimit-Usage")
            .and_then(|v| v.to_str().ok());

        if let (Some(limit_str), Some(usage_str)) = (limit, usage) {
            let parts_limit: Vec<&str> = limit_str.split(',').collect();
            let parts_usage: Vec<&str> = usage_str.split(',').collect();

            if parts_limit.len() == 2 && parts_usage.len() == 2 {
                let mut rl = self.rate_limit.write().await;
                rl.limit_15min = parts_limit[0].trim().parse().unwrap_or(0);
                rl.limit_daily = parts_limit[1].trim().parse().unwrap_or(0);
                rl.usage_15min = parts_usage[0].trim().parse().unwrap_or(0);
                rl.usage_daily = parts_usage[1].trim().parse().unwrap_or(0);
                rl.updated_at = Utc::now();
                debug!(
                    "Rate limit updated: {}/{} (15min), {}/{} (daily)",
                    rl.usage_15min, rl.limit_15min, rl.usage_daily, rl.limit_daily
                );
            }
        }
    }

    pub async fn get_access_token(
        &self,
        athlete: &Athlete,
    ) -> Result<String> {
        let athlete_id = athlete.id;
        if let Some(access_token) = self.get_token_from_cache(athlete_id).await {
            debug!("Using cached access token for athlete: {athlete_id}");
            Ok(access_token)
        } else {
            debug!(
                "Access token not in cache, refreshing for athlete: {athlete_id}"
            );
            let refreshed_token = StravaClient::refresh_token(
                self.client_id,
                &self.client_secret,
                &athlete.refresh_token,
            )
            .await?;

            self.cache_access_token(athlete_id, &refreshed_token.access_token)
                .await;

            AthleteRepository::update_refresh_token(
                &self.db_pools,
                athlete_id,
                &refreshed_token.refresh_token,
            )
            .await?;
            Ok(refreshed_token.access_token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_app_state() -> Arc<AppState> {
        let pool = Pool::<Sqlite>::connect("sqlite::memory:").await.unwrap();
        AppState::new(12345, "secret".to_string(), pool)
    }

    #[tokio::test]
    async fn cache_and_retrieve_access_token() {
        let state = test_app_state().await;
        state.cache_access_token(1, "token_abc").await;
        let cached = state.get_token_from_cache(1).await;
        assert_eq!(cached, Some("token_abc".to_string()));
    }

    #[tokio::test]
    async fn get_token_returns_none_when_not_cached() {
        let state = test_app_state().await;
        let cached = state.get_token_from_cache(999).await;
        assert_eq!(cached, None);
    }

    #[tokio::test]
    async fn invalidate_removes_cached_token() {
        let state = test_app_state().await;
        state.cache_access_token(2, "token_xyz").await;
        state.invalidate_access_token(2).await;
        let cached = state.get_token_from_cache(2).await;
        assert_eq!(cached, None);
    }

    #[tokio::test]
    async fn separate_athletes_have_separate_caches() {
        let state = test_app_state().await;
        state.cache_access_token(1, "token_a").await;
        state.cache_access_token(2, "token_b").await;
        assert_eq!(state.get_token_from_cache(1).await, Some("token_a".to_string()));
        assert_eq!(state.get_token_from_cache(2).await, Some("token_b".to_string()));
    }
}
