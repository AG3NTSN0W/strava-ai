use crate::libs::StravAIError;
use crate::libs::models::strava::activity::Activity;
use crate::libs::models::strava::token::{RefreshToken, Token};
use crate::libs::models::strava::update_activity::UpdateActivity;
use chrono::Utc;
use log::debug;

const STRAVA_BASE_URL: &str = "https://www.strava.com/api/v3";
const STRAVA_AUTH_URL: &str = "https://www.strava.com/oauth/token";

#[derive(Debug)]
pub struct StravaClient {}

impl StravaClient {
    fn get_today_timestamp() -> i64 {
        let now = Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        today_start.timestamp()
    }

    pub async fn get_activities(
        access_token: &str,
        after: i64,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Fetching activities with access token: {access_token} and after timestamp: {after}"
        );
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/athlete/activities?after={after}");

        let response = client.get(&url).bearer_auth(access_token).send().await?;

        let activities = response.json::<Vec<Activity>>().await?;
        Ok(activities)
    }

    pub async fn get_activities_for_today(
        access_token: &str,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        let after = Self::get_today_timestamp();
        Self::get_activities(access_token, after).await
    }

    pub async fn get_all_activities(
        access_token: &str,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Fetching all activities");
        let per_page = 200;
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/athlete/activities?per_page={per_page}");

        let response = client.get(&url).bearer_auth(access_token).send().await?;

        let activities = response.json::<Vec<Activity>>().await?;
        Ok(activities)
    }

    pub async fn exchange_authorization_code(
        client_id: i32,
        client_secret: &str,
        code: &str,
    ) -> Result<Token, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Exchanging authorization code for token for client_id: {client_id}");
        let client = reqwest::Client::new();
        let url = format!(
            "{STRAVA_AUTH_URL}?client_id={client_id}&client_secret={client_secret}&code={code}&grant_type=authorization_code"
        );

        debug!("Exchanging authorization code for token with URL: {url}");

        let response = client.post(&url).send().await?;

        let token_response = response.json::<Token>().await?;
        Ok(token_response)
    }

    pub async fn refresh_token(
        client_id: i32,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<RefreshToken, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Refreshing token for client_id: {client_id}");
        let client = reqwest::Client::new();
        let url = format!(
            "{STRAVA_AUTH_URL}?client_id={client_id}&client_secret={client_secret}&grant_type=refresh_token&refresh_token={refresh_token}"
        );

        let response = client.post(&url).send().await?;

        let token_response = response.json::<RefreshToken>().await?;
        Ok(token_response)
    }

    pub async fn update_activity(
        access_token: &str,
        activity_id: u64,
        update: &UpdateActivity,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Updating activity for client_id: {activity_id}. Update: {update:?}"
        );
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/activities/{activity_id}");

        let response = client
            .put(&url)
            .bearer_auth(access_token)
            .json(update)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(Box::new(StravAIError("Failed to update activity".into())))
    }
}
