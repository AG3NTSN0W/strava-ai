use crate::libs::DEFAULT_PROMPT;
use crate::libs::models::strava::update_activity::UpdateActivity;
use crate::libs::models::{Athlete, AthleteActivity};
use crate::libs::repository::{ActivityRepository, AthleteRepository};
use crate::{AppState, libs::strava_client::StravaClient};
use axum::{
    Form,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use log::{debug, error, info};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ExchangeTokenQuery {
    code: Option<String>,
    #[allow(dead_code)]
    scope: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateActivityRequest {
    pub athlete_id: i64,
    pub activity_id: u64,
    pub description: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSettingsRequest {
    pub athlete_id: i64,
    pub prompt: String,
    pub auto_update: Option<bool>,
}

pub async fn exchange_token(
    Query(params): Query<ExchangeTokenQuery>,
    State(app_state): State<Arc<AppState>>,
) -> Response {
    // Check if authorization was denied by user
    if params.error.is_some() {
        return (StatusCode::BAD_REQUEST, "Authorization denied by user.").into_response();
    }

    // Extract the authorization code
    let code = match params.code {
        Some(c) => c,
        None => {
            error!("Missing authorization code in callback");
            return (StatusCode::BAD_REQUEST, "Missing authorization code.").into_response();
        }
    };

    // Exchange the temporary code for an access token
    match StravaClient::exchange_authorization_code(
        app_state.client_id,
        &app_state.client_secret,
        &code,
    )
    .await
    {
        Ok(token) => {
            let athlete_name = &token.athlete.firstname;
            info!("Authenticated athlete: {athlete_name}");

            let athlete_id = token.athlete.id;
            let access_token = token.access_token.clone();

            let athlete = Athlete {
                id: athlete_id,
                firstname: token.athlete.firstname,
                lastname: token.athlete.lastname,
                premium: token.athlete.premium,
                refresh_token: token.refresh_token,
                auto_update: false,
                prompt: DEFAULT_PROMPT.to_string(),
            };

            if let Ok(exists) = AthleteRepository::exists(&app_state.db_pools, athlete_id).await {
                if exists {
                    match AthleteRepository::update_refresh_token(
                        &app_state.db_pools,
                        athlete_id,
                        &athlete.refresh_token,
                    )
                    .await
                    {
                        Ok(_) => {
                            debug!("Update Refresh Token for Id: {athlete_id}")
                        }
                        Err(e) => {
                            error!("Failed to Update Refresh Token: {e}");
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add athlete")
                                .into_response();
                        }
                    }
                } else {
                    match AthleteRepository::create(&app_state.db_pools, &athlete).await {
                        Ok(_) => {
                            debug!("Created new athlete with id: {athlete_id}")
                        }
                        Err(e) => {
                            error!("Failed to save athlete to database: {e}");
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add athlete")
                                .into_response();
                        }
                    }
                }
            } else {
                return (StatusCode::BAD_REQUEST, "Missing authorization code.").into_response();
            }

            // Cache the access token with a 30min TTL
            app_state
                .cache_access_token(athlete_id, &access_token)
                .await;
            debug!("Cached access token for athlete: {athlete_id}");

            let activities = StravaClient::get_all_activities(&token.access_token)
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to fetch activities for athlete {athlete_id}: {e}");
                    vec![]
                });
            debug!("All activities: {:#?}", activities.len());
            if !activities.is_empty() {
                for activity in activities {
                    match ActivityRepository::exists(&app_state.db_pools, activity.id).await {
                        Ok(exists) => {
                            if exists {
                                debug!("Activity with id: {} found", activity.id);
                                continue;
                            }
                        }
                        Err(e) => debug!("No activity with id: {}: Error: {e}", activity.id),
                    }
                    let athlete_activity = AthleteActivity::from((activity, athlete.id));
                    debug!(
                        "Adding athlete id: {athlete_id}, activity: {athlete_activity:#?}"
                    );
                    ActivityRepository::create(&app_state.db_pools, &athlete_activity)
                        .await
                        .unwrap_or_else(|e| {
                            error!("Failed to save activities to database for athlete {athlete_id}: {e}");
                        });
                }
            }

            Redirect::to("/stravai").into_response()
        }
        Err(err) => {
            error!("Failed to exchange token: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to exchange token.",
            )
                .into_response()
        }
    }
}

pub async fn update_activity(
    State(app_state): State<Arc<AppState>>,
    Form(update): Form<UpdateActivityRequest>,
) -> impl IntoResponse {
    debug!("Update activity: {update:#?}");
    let athlete = match AthleteRepository::get_by_id(&app_state.db_pools, update.athlete_id).await {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to fetch athlete from database: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Athlete not found").into_response();
        }
    };

    let athlete = match athlete {
        Some(a) => a,
        None => {
            error!("Athlete not found");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Athlete not found").into_response();
        }
    };

    let access_token = match app_state.get_access_token(&athlete).await {
        Ok(at) => at,
        Err(e) => {
            error!("Failed to get access token: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Access token not found").into_response();
        }
    };

    let activity = UpdateActivity {
        commute: None,
        trainer: None,
        name: Some(update.name.to_string()),
        activity_type: None,
        sport_type: None,
        description: Some(update.description.to_string()),
        hide_from_home: None,
        gear_id: None,
    };
    match StravaClient::update_activity(&access_token, update.activity_id, &activity).await {
        Ok(_) => (StatusCode::OK, "").into_response(),
        Err(e) => {
            error!("Failed to update activity: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update activity",
            )
                .into_response();
        }
    };

    match ActivityRepository::update_description_and_name(
        &app_state.db_pools,
        &update.description,
        &update.name,
        update.activity_id as i64,
    )
    .await
    {
        Ok(_) => {
            info!("Activity Updated: activity id: {}", update.activity_id);
            (StatusCode::OK, "").into_response()
        }
        Err(e) => {
            error!("Failed to update activity: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update activity",
            )
                .into_response()
        }
    }
}

pub async fn update_settings(
    State(app_state): State<Arc<AppState>>,
    Form(update): Form<UpdateSettingsRequest>,
) -> impl IntoResponse {
    let auto_update = update.auto_update.unwrap_or(false);
    debug!(
        "Update athlete settings: {update:?}, auto_update: {auto_update}"
    );
    match AthleteRepository::update_settings(
        &app_state.db_pools,
        update.athlete_id,
        &update.prompt,
        auto_update,
    )
    .await
    {
        Ok(_) => (StatusCode::OK, "Update was successfully").into_response(),
        Err(e) => {
            error!("Failed to fetch athlete from database: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to update settings",
            )
                .into_response()
        }
    }
}
