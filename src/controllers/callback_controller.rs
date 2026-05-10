use crate::libs::DEFAULT_PROMPT;
use crate::libs::models::activity_stream::ActivityStream;
use crate::libs::models::strava::update_activity::UpdateActivity;
use crate::libs::models::{Athlete, AthleteActivity};
use crate::libs::repository::{ActivityRepository, ActivityStreamRepository, AthleteRepository};
use crate::{AppState, libs::strava_client::StravaClient};
use axum::{
    Form,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use log::{debug, error, info};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
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

#[derive(Debug, Deserialize)]
pub struct BackfillStreamsQuery {
    pub athlete_id: i64,
}

// --- Handlers ---

pub async fn exchange_token(
    Query(params): Query<ExchangeTokenQuery>,
    State(app_state): State<Arc<AppState>>,
) -> Response {
    if params.error.is_some() {
        return (StatusCode::BAD_REQUEST, "Authorization denied by user.").into_response();
    }

    let code = match params.code {
        Some(c) => c,
        None => {
            error!("Missing authorization code in callback");
            return (StatusCode::BAD_REQUEST, "Missing authorization code.").into_response();
        }
    };

    let token = match StravaClient::exchange_authorization_code(
        app_state.client_id,
        &app_state.client_secret,
        &code,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to exchange token: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to exchange token.").into_response();
        }
    };

    let athlete_id = token.athlete.id;
    let access_token = token.access_token.clone();
    info!("Authenticated athlete: {}", token.athlete.firstname);

    let athlete = Athlete {
        id: athlete_id,
        firstname: token.athlete.firstname,
        lastname: token.athlete.lastname,
        premium: token.athlete.premium,
        refresh_token: token.refresh_token,
        auto_update: false,
        prompt: DEFAULT_PROMPT.to_string(),
    };

    if let Err(e) = upsert_athlete(&app_state.db_pools, &athlete).await {
        error!("Failed to upsert athlete: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save athlete").into_response();
    }

    app_state
        .cache_access_token(athlete_id, &access_token)
        .await;

    import_activities(&app_state, athlete_id, &access_token).await;

    Redirect::to("/stravai").into_response()
}

pub async fn update_activity(
    State(app_state): State<Arc<AppState>>,
    Form(update): Form<UpdateActivityRequest>,
) -> impl IntoResponse {
    debug!("Update activity: {update:#?}");

    let athlete = match get_athlete(&app_state.db_pools, update.athlete_id).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };

    let access_token = match app_state.get_access_token(&athlete).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to get access token: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Access token error").into_response();
        }
    };

    let strava_update = UpdateActivity {
        name: Some(update.name.clone()),
        description: Some(update.description.clone()),
        ..Default::default()
    };

    if let Err(e) =
        StravaClient::update_activity(&access_token, update.activity_id, &strava_update).await
    {
        error!("Failed to update activity on Strava: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update activity",
        )
            .into_response();
    }

    if let Err(e) = ActivityRepository::update_description_and_name(
        &app_state.db_pools,
        &update.description,
        &update.name,
        update.activity_id as i64,
    )
    .await
    {
        error!("Failed to update activity in database: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update activity",
        )
            .into_response();
    }

    info!("Activity updated: {}", update.activity_id);
    (StatusCode::OK, "").into_response()
}

pub async fn update_settings(
    State(app_state): State<Arc<AppState>>,
    Form(update): Form<UpdateSettingsRequest>,
) -> impl IntoResponse {
    let auto_update = update.auto_update.unwrap_or(false);
    debug!("Update settings for athlete {}: auto_update={auto_update}", update.athlete_id);

    match AthleteRepository::update_settings(
        &app_state.db_pools,
        update.athlete_id,
        &update.prompt,
        auto_update,
    )
    .await
    {
        Ok(_) => (StatusCode::OK, "Update was successful").into_response(),
        Err(e) => {
            error!("Failed to update settings: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Unable to update settings").into_response()
        }
    }
}

pub async fn backfill_streams(
    Query(params): Query<BackfillStreamsQuery>,
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let athlete = match AthleteRepository::get_by_id(&app_state.db_pools, params.athlete_id).await {
        Ok(Some(a)) => a,
        _ => return (StatusCode::NOT_FOUND, "Athlete not found").into_response(),
    };

    let access_token = match app_state.get_access_token(&athlete).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to get access token: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Access token error").into_response();
        }
    };

    let activities =
        ActivityRepository::get_past_month_by_athlete_id(&app_state.db_pools, params.athlete_id)
            .await
            .unwrap_or_default();

    let count = activities.len();

    let activity_ids: Vec<i64> = activities.into_iter().map(|a| a.id).collect();
    spawn_stream_fetch(app_state.db_pools.clone(), access_token, activity_ids);

    (StatusCode::OK, format!("Backfilling streams for {count} activities")).into_response()
}

// --- Helpers ---

async fn upsert_athlete(
    pool: &Pool<Sqlite>,
    athlete: &Athlete,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if AthleteRepository::exists(pool, athlete.id).await? {
        AthleteRepository::update_refresh_token(pool, athlete.id, &athlete.refresh_token).await?;
        debug!("Updated refresh token for athlete: {}", athlete.id);
    } else {
        AthleteRepository::create(pool, athlete).await?;
        debug!("Created new athlete: {}", athlete.id);
    }
    Ok(())
}

async fn get_athlete(pool: &Pool<Sqlite>, athlete_id: i64) -> Result<Athlete, Response> {
    match AthleteRepository::get_by_id(pool, athlete_id).await {
        Ok(Some(a)) => Ok(a),
        Ok(None) => {
            error!("Athlete {athlete_id} not found");
            Err((StatusCode::NOT_FOUND, "Athlete not found").into_response())
        }
        Err(e) => {
            error!("Failed to fetch athlete {athlete_id}: {e}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())
        }
    }
}

async fn import_activities(app_state: &AppState, athlete_id: i64, access_token: &str) {
    let activities = StravaClient::get_all_activities(access_token)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to fetch activities for athlete {athlete_id}: {e}");
            vec![]
        });

    debug!("Importing {} activities for athlete {athlete_id}", activities.len());

    let mut new_activity_ids = Vec::new();

    for activity in activities {
        let exists = ActivityRepository::exists(&app_state.db_pools, activity.id)
            .await
            .unwrap_or(false);
        if exists {
            continue;
        }

        let activity_id = activity.id;
        let athlete_activity = AthleteActivity::from((activity, athlete_id));

        if let Err(e) = ActivityRepository::create(&app_state.db_pools, &athlete_activity).await {
            error!("Failed to save activity {activity_id}: {e}");
            continue;
        }

        new_activity_ids.push(activity_id);
    }

    if !new_activity_ids.is_empty() {
        spawn_stream_fetch(app_state.db_pools.clone(), access_token.to_string(), new_activity_ids);
    }
}

fn spawn_stream_fetch(pool: Pool<Sqlite>, access_token: String, activity_ids: Vec<i64>) {
    tokio::spawn(async move {
        for activity_id in activity_ids {
            let streams = match StravaClient::get_activity_streams(&access_token, activity_id).await
            {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to fetch streams for activity {activity_id}: {e}");
                    continue;
                }
            };

            for stream in streams {
                let activity_stream = ActivityStream {
                    id: 0,
                    activity_id,
                    stream_type: stream.stream_type,
                    data: stream.data.to_string(),
                    series_type: stream.series_type,
                    original_size: stream.original_size,
                    resolution: stream.resolution,
                };
                if let Err(e) = ActivityStreamRepository::create(&pool, &activity_stream).await {
                    error!("Failed to save stream for activity {activity_id}: {e}");
                }
            }
        }
    });
}
