use crate::AppState;
use crate::libs::models::strava::activity::Activity;
use crate::libs::models::strava::stream::StreamResponse;
use crate::libs::models::strava::token::{RefreshToken, Token};
use crate::libs::models::strava::update_activity::UpdateActivity;
use chrono::Utc;
use log::{debug, error};
use reqwest::StatusCode;
use thiserror::Error;

const STRAVA_BASE_URL: &str = "https://www.strava.com/api/v3";
const STRAVA_AUTH_URL: &str = "https://www.strava.com/oauth/token";

#[derive(Error, Debug)]
pub enum StravaClientError {
    #[error("Invalid header (expected {expected:?}, got {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("Missing attribute: {0}")]
    MissingAttribute(String),

    #[error("Strava API error {context}: HTTP status {status_code}")]
    UnknowError {
        context: String,
        status_code: StatusCode,
    },

    #[error("Unauthorized: Client {0}")]
    UNAUTHORIZED(i64),

    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Strava authorization error: HTTP status {status_code}, Content: {context}")]
    FailedToAuthorized {
        context: String,
        status_code: StatusCode,
    },

    #[error("Strava token refresh error: HTTP status {status_code}, Content: {context}")]
    FailedToRefreshToken {
        context: String,
        status_code: StatusCode,
    },
}

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
        athlete_id: i64,
    ) -> Result<reqwest::Response, StravaClientError> {
        app_state.update_rate_limit(&response).await;
        let status_code = response.status();
        if response.status().is_success() {
            return Ok(response);
        }

        if status_code == StatusCode::UNAUTHORIZED {
            app_state.invalidate_access_token(athlete_id).await;
            return Err(StravaClientError::UNAUTHORIZED(athlete_id));
        }

        Err(StravaClientError::UnknowError {
            status_code,
            context: context.to_owned(),
        })
    }

    pub async fn get_activities(
        access_token: &str,
        after: i64,
        app_state: &AppState,
        athlete_id: i64,
    ) -> Result<Vec<Activity>, StravaClientError> {
        debug!("Fetching activities after timestamp: {after}");
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/athlete/activities?after={after}");

        let response = client.get(&url).bearer_auth(access_token).send().await?;
        let response =
            Self::check_response(response, "get_activities", app_state, athlete_id).await?;

        let activities = response.json::<Vec<Activity>>().await?;
        Ok(activities)
    }

    pub async fn get_activities_for_today(
        access_token: &str,
        app_state: &AppState,
        athlete_id: i64,
    ) -> Result<Vec<Activity>, StravaClientError> {
        let after = Self::get_today_timestamp();
        Self::get_activities(access_token, after, app_state, athlete_id).await
    }

    pub async fn get_all_activities(
        access_token: &str,
        app_state: &AppState,
        athlete_id: i64,
    ) -> Result<Vec<Activity>, StravaClientError> {
        debug!("Fetching all activities");
        let per_page = 100;
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/athlete/activities?per_page={per_page}");

        let response = client.get(&url).bearer_auth(access_token).send().await?;
        let response =
            Self::check_response(response, "get_all_activities", app_state, athlete_id).await?;

        let activities = response.json::<Vec<Activity>>().await?;
        Ok(activities)
    }

    pub async fn exchange_authorization_code(
        client_id: i32,
        client_secret: &str,
        code: &str,
    ) -> Result<Token, StravaClientError> {
        debug!("Exchanging authorization code for token for client_id: {client_id}");
        let client = reqwest::Client::new();
        let url = format!(
            "{STRAVA_AUTH_URL}?client_id={client_id}&client_secret={client_secret}&code={code}&grant_type=authorization_code"
        );

        let response = client.post(&url).send().await?;
        if !response.status().is_success() {
            let status_code = response.status();
            let context = response.text().await.unwrap_or_default();
            return Err(StravaClientError::FailedToAuthorized {
                status_code,
                context,
            });
        }

        let token_response = response.json::<Token>().await?;
        Ok(token_response)
    }

    pub async fn refresh_token(
        client_id: i32,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<RefreshToken, StravaClientError> {
        debug!("Refreshing token for client_id: {client_id}");
        let client = reqwest::Client::new();
        let url = format!(
            "{STRAVA_AUTH_URL}?client_id={client_id}&client_secret={client_secret}&grant_type=refresh_token&refresh_token={refresh_token}"
        );

        let response = client.post(&url).send().await?;
        if !response.status().is_success() {
            let status_code = response.status();
            let context = response.text().await.unwrap_or_default();
            return Err(StravaClientError::FailedToRefreshToken {
                status_code,
                context,
            });
        }

        let token_response = response.json::<RefreshToken>().await?;
        Ok(token_response)
    }

    pub async fn update_activity(
        access_token: &str,
        activity_id: u64,
        update: &UpdateActivity,
        app_state: &AppState,
        athlete_id: i64,
    ) -> Result<(), StravaClientError> {
        debug!("Updating activity: {activity_id}. Update: {update:?}");
        let client = reqwest::Client::new();
        let url = format!("{STRAVA_BASE_URL}/activities/{activity_id}");

        let response = client
            .put(&url)
            .bearer_auth(access_token)
            .json(update)
            .send()
            .await?;

        Self::check_response(
            response,
            &format!("update_activity {activity_id}"),
            app_state,
            athlete_id,
        )
        .await?;
        Ok(())
    }

    pub async fn get_activity_streams(
        access_token: &str,
        activity_id: i64,
        app_state: &AppState,
        athlete_id: i64,
    ) -> Result<Vec<StreamResponse>, StravaClientError> {
        debug!("Fetching streams for activity: {activity_id}");
        let client = reqwest::Client::new();
        let url = format!(
            "{STRAVA_BASE_URL}/activities/{activity_id}/streams?keys=distance,heartrate,cadence,latlng,altitude,velocity_smooth"
        );

        let response = client.get(&url).bearer_auth(access_token).send().await?;
        let response = Self::check_response(
            response,
            &format!("get_activity_streams {activity_id}"),
            app_state,
            athlete_id,
        )
        .await?;

        let streams = response.json::<Vec<StreamResponse>>().await?;
        Ok(streams)
    }
}
