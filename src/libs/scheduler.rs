use crate::AppState;
use crate::libs::models::AthleteActivity;
use crate::libs::models::activity_stream::ActivityStream;
use crate::libs::models::strava::activity::Activity;
use crate::libs::models::strava::update_activity::UpdateActivity;
use crate::libs::ollama_client::OllamaClient;
use crate::libs::repository::{ActivityRepository, ActivityStreamRepository, AthleteRepository};
use crate::libs::strava_client::StravaClient;
use log::{debug, info, warn};
use std::env;
use std::sync::Arc;
use tokio::time::{Duration, interval};

fn get_internal_hours() -> f64 {
    env::var("STRAVA_INTERVAL")
        .ok()
        .and_then(|val| {
            val.trim().trim_matches('"').parse().ok().or_else(|| {
                warn!("STRAVA_INTERVAL value '{val}' is not a valid number, using default");
                None
            })
        })
        .unwrap_or(5.0)
}

/// Start the background scheduler task
pub async fn start_scheduler(app_state: Arc<AppState>) {
    tokio::spawn(async move {
        let strava_interval = get_internal_hours() * 60.0 * 60.0;
        let mut interval = interval(Duration::from_secs(strava_interval as u64));

        loop {
            interval.tick().await;
            debug!("Running scheduled task at {}", chrono::Utc::now());

            if let Err(e) = run_scheduled_task(&app_state).await {
                log::error!("Scheduler task failed: {e}");
            }
        }
    });
}

/// Main scheduled task that runs every 5 hours
async fn run_scheduled_task(
    app_state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !app_state.rate_limit.read().await.has_budget() {
        warn!("Skipping scheduled task: rate limit budget exhausted");
        return Ok(());
    }

    let athletes = AthleteRepository::get_all(&app_state.db_pools).await?;

    for athlete in athletes {
        let athlete_name = format!("{} {}", athlete.firstname, athlete.lastname);

        let access_token = match app_state.get_access_token(&athlete).await {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to get access token for {athlete_name}: {e}");
                continue;
            }
        };

        if !athlete.auto_update {
            debug!(
                "Summaries generation skipped for athlete: {athlete_name}"
            );

            if let Err(e) = add_activities_to_db(app_state, athlete.id, &access_token).await {
                log::error!("Failed to add activities for {athlete_name}: {e}");
            }
            continue;
        }

        debug!("Generate Summaries for athlete: {athlete_name}");

        if let Err(e) =
            generate_summaries(athlete.id, &access_token, app_state, athlete.prompt).await
        {
            log::error!("Failed to generate summaries for {athlete_name}: {e}");
        }
    }
    Ok(())
}

async fn add_activities_to_db(
    app_state: &AppState,
    athlete_id: i64,
    access_token: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let activities: Vec<Activity> = StravaClient::get_activities_for_today(access_token, app_state, athlete_id).await?;
    for activity in activities {
        let name = activity.name.to_string();
        update_activities_table(app_state, athlete_id, &activity, &name, "").await?;
        fetch_and_save_streams(app_state, access_token, activity.id, athlete_id).await?;
    }
    Ok(())
}

async fn update_activities_table(
    app_state: &AppState,
    athlete_id: i64,
    activity: &Activity,
    name: &str,
    description: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let athlete_activity = AthleteActivity {
        id: activity.id,
        athlete_id,
        name: name.to_string(),
        description: description.to_string(),
        distance: activity.distance,
        moving_time: activity.moving_time.clone(),
        elapsed_time: activity.elapsed_time.clone(),
        total_elevation_gain: activity.total_elevation_gain,
        activity_type: activity.activity_type.clone(),
        sport_type: activity.sport_type.clone(),
        start_date_local: activity.start_date_local.clone(),
        achievement_count: activity.achievement_count,
        average_speed: activity.average_speed,
        max_speed: activity.max_speed,
        average_watts: activity.average_watts,
        kilojoules: activity.kilojoules,
        average_heartrate: activity.average_heartrate,
        max_heartrate: activity.max_heartrate,
        elev_high: activity.elev_high,
        elev_low: activity.elev_low,
        pr_count: activity.pr_count,
    };

    ActivityRepository::create(&app_state.db_pools, &athlete_activity).await?;

    Ok(())
}

async fn generate_summaries(
    athlete_id: i64,
    access_token: &str,
    app_state: &AppState,
    prompt: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let activities: Vec<Activity> = StravaClient::get_activities_for_today(access_token, app_state, athlete_id).await?;
    for activity in activities {
        if ActivityRepository::exists(&app_state.db_pools, activity.id).await? {
            continue;
        }

        let summary = OllamaClient::generate_activity_summary(&activity, &prompt).await?;

        update_activities_table(
            app_state,
            athlete_id,
            &activity,
            &summary.title,
            &summary.description,
        )
        .await?;

        StravaClient::update_activity(
            access_token,
            activity.id as u64,
            &UpdateActivity {
                commute: None,
                trainer: None,
                name: Some(summary.title.clone()),
                activity_type: None,
                sport_type: None,
                description: Some(summary.description.clone()),
                hide_from_home: None,
                gear_id: None,
            },
            app_state,
            athlete_id
        )
        .await?;

        info!("Activity updated. athlete_id: {athlete_id}, Activity Id: {}", activity.id);

        fetch_and_save_streams(app_state, access_token, activity.id, athlete_id).await?;
    }

    Ok(())
}

async fn fetch_and_save_streams(
    app_state: &AppState,
    access_token: &str,
    activity_id: i64,
    athlete_id: i64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let existing = ActivityStreamRepository::get_by_activity_id(&app_state.db_pools, activity_id).await?;
    if !existing.is_empty() {
        return Ok(());
    }

    let streams = StravaClient::get_activity_streams(access_token, activity_id, app_state, athlete_id).await?;

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
        ActivityStreamRepository::create(&app_state.db_pools, &activity_stream).await?;
    }

    Ok(())
}
