use crate::AppState;
use crate::libs::StravAIError;
use crate::libs::models::strava::activity::Activity;
use crate::libs::models::strava::stream::StreamResponse;
use crate::libs::models::strava::token::{RefreshToken, Token};
use crate::libs::models::strava::update_activity::UpdateActivity;
use chrono::Utc;
use log::{debug, error};

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

    /// Checks response status, updates rate limit, and returns a descriptive error if not successful.
    async fn check_response(
        response: reqwest::Response,
        context: &str,
        app_state: &AppState,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        app_state.update_rate_limit(&response).await;

        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Strava API error [{context}]: {status} - {body}");
        Err(Box::new(StravAIError(format!(
            "{context}: HTTP {status} - {body}"
        ))))
    }

    pub async fn get_activities(
        access_token: &str,
        after: i64,
        app_state: &AppState,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Fetching activities after timestamp: {after}");
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/athlete/activities?after={after}");

        let response = client.get(&url).bearer_auth(access_token).send().await?;
        let response = Self::check_response(response, "get_activities", app_state).await?;

        let activities = response.json::<Vec<Activity>>().await?;
        Ok(activities)
    }

    pub async fn get_activities_for_today(
        access_token: &str,
        app_state: &AppState,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        let after = Self::get_today_timestamp();
        Self::get_activities(access_token, after, app_state).await
    }

    pub async fn get_all_activities(
        access_token: &str,
        app_state: &AppState,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Fetching all activities");
        let per_page = 100;
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/athlete/activities?per_page={per_page}");

        let response = client.get(&url).bearer_auth(access_token).send().await?;
        let response = Self::check_response(response, "get_all_activities", app_state).await?;

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

        let response = client.post(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Strava API error [exchange_authorization_code]: {status} - {body}");
            return Err(Box::new(StravAIError(format!(
                "exchange_authorization_code: HTTP {status} - {body}"
            ))));
        }

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
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Strava API error [refresh_token]: {status} - {body}");
            return Err(Box::new(StravAIError(format!(
                "refresh_token: HTTP {status} - {body}"
            ))));
        }

        let token_response = response.json::<RefreshToken>().await?;
        Ok(token_response)
    }

    pub async fn update_activity(
        access_token: &str,
        activity_id: u64,
        update: &UpdateActivity,
        app_state: &AppState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Updating activity: {activity_id}. Update: {update:?}");
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/activities/{activity_id}");

        let response = client
            .put(&url)
            .bearer_auth(access_token)
            .json(update)
            .send()
            .await?;

        Self::check_response(response, &format!("update_activity {activity_id}"), app_state).await?;
        Ok(())
    }

    pub async fn get_activity_streams(
        access_token: &str,
        activity_id: i64,
        app_state: &AppState,
    ) -> Result<Vec<StreamResponse>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Fetching streams for activity: {activity_id}");
        let client = reqwest::Client::new();
        let url = format!(
            "{STRAVA_BASE_URL}/activities/{activity_id}/streams?keys=distance,heartrate,cadence,latlng,altitude,velocity_smooth"
        );

        let response = client.get(&url).bearer_auth(access_token).send().await?;
        let response =
            Self::check_response(response, &format!("get_activity_streams {activity_id}"), app_state).await?;

        let streams = response.json::<Vec<StreamResponse>>().await?;
        Ok(streams)
    }
}
