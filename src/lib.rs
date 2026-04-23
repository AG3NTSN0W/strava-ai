use crate::libs::models::Athlete;
use crate::libs::repository::AthleteRepository;
use crate::libs::strava_client::StravaClient;
use log::debug;
use moka::future::Cache;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use std::time::Duration;

pub mod controllers;
pub mod libs;

#[derive(Clone)]
pub struct AppState {
    db_pools: Pool<Sqlite>,
    client_id: i32,
    client_secret: String,
    cache: Arc<Cache<String, String>>,
}

impl AppState {
    pub fn new(client_id: i32, client_secret: String, db_pools: Pool<Sqlite>) -> Arc<Self> {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(5 * 60 * 60)) // 5 hours
            .build();

        Arc::new(Self {
            db_pools: db_pools.clone(),
            client_id,
            client_secret,
            cache: Arc::new(cache),
        })
    }

    /// Get cached access token for an athlete
    async fn get_token_from_cache(&self, athlete_id: i64) -> Option<String> {
        let cache_key = format!("access_token_{athlete_id}");
        self.cache.get(&cache_key).await
    }

    /// Cache access token for an athlete (5-hour TTL)
    async fn cache_access_token(&self, athlete_id: i64, token: &str) {
        let cache_key = format!("access_token_{athlete_id}");
        self.cache.insert(cache_key, token.to_string()).await;
    }

    pub async fn get_access_token(
        &self,
        athlete: &Athlete,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
