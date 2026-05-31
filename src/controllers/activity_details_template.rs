use super::{HtmlTemplate, TemplateQueryParams};
use crate::AppState;
use crate::libs::DEFAULT_PROMPT;
use crate::libs::models::AthleteActivity;
use crate::libs::repository::{ActivityRepository, ActivityStreamRepository, AthleteRepository};
use askama::Template;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use log::error;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "activity_details_template.html")]
pub struct ActivityDetailsTemplate {
    activity: Option<AthleteActivity>,
    prompt: String,
    map_points: Vec<Vec<f64>>,
    heartrate_stream: Vec<f64>,
    altitude_stream: Vec<f64>,
    velocity_stream: Vec<f64>,
}

impl ActivityDetailsTemplate {
    pub async fn new(
        State(state): State<Arc<AppState>>,
        query_params: TemplateQueryParams,
    ) -> Self {
        let activity_id = query_params.activity_id.unwrap();
        let activity = ActivityRepository::get_by_id(&state.db_pools, activity_id)
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Failed to fetch activities from database for athlete {activity_id}: {e}"
                );
                None
            });

        match activity {
            None => Self {
                activity: None,
                prompt: DEFAULT_PROMPT.to_string(),
                map_points: Vec::new(),
                heartrate_stream: Vec::new(),
                altitude_stream: Vec::new(),
                velocity_stream: Vec::new(),
            },
            Some(activity) => Self::get_activity(&state.db_pools, &query_params, activity).await,
        }
    }

    async fn get_activity(
        pool: &Pool<Sqlite>,
        query_params: &TemplateQueryParams,
        activity: AthleteActivity,
    ) -> ActivityDetailsTemplate {
        let activity_id = if let Some(athlete) = query_params.activity_id {
            athlete
        } else {
            return Self {
                activity: Some(activity),
                prompt: DEFAULT_PROMPT.to_string(),
                map_points: Vec::new(),
                heartrate_stream: Vec::new(),
                altitude_stream: Vec::new(),
                velocity_stream: Vec::new(),
            };
        };

        let activity = AthleteActivity {
            start_date_local: activity
                .start_date_local
                .parse::<DateTime<Utc>>()
                .map(|dt| dt.format("%d %B %Y %H:%M:%S").to_string())
                .unwrap_or_else(|_| activity.start_date_local.clone()),
            ..activity.clone()
        };

        let athlete_id = query_params.athlete_id;
        let athlete = AthleteRepository::get_by_id(pool, athlete_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to fetch athletes from database: {e}");
                None
            });

        let map_points =
            ActivityStreamRepository::get_latlng_points_by_id(pool, athlete_id, activity_id)
                .await
                .unwrap_or_else(|e| {
                    error!("Unable to get map points: {e}");
                    vec![]
                });

        let heartrate_stream =
            ActivityStreamRepository::get_metric_stream_by_activity_id(pool, activity_id, "heartrate")
                .await
                .unwrap_or_default();

        let altitude_stream =
            ActivityStreamRepository::get_metric_stream_by_activity_id(pool, activity_id, "altitude")
                .await
                .unwrap_or_default();

        let velocity_stream =
            ActivityStreamRepository::get_metric_stream_by_activity_id(pool, activity_id, "velocity_smooth")
                .await
                .unwrap_or_default();

        let prompt = athlete
            .map(|a| a.prompt)
            .unwrap_or_else(|| DEFAULT_PROMPT.to_string());

        Self {
            activity: Some(activity),
            prompt,
            map_points,
            heartrate_stream,
            altitude_stream,
            velocity_stream,
        }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    query_params: Query<TemplateQueryParams>,
) -> impl IntoResponse {
    HtmlTemplate(ActivityDetailsTemplate::new(state, query_params.0).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use askama::Template;

    fn test_activity() -> AthleteActivity {
        AthleteActivity {
            id: 123,
            athlete_id: 1,
            name: "Morning Run".to_string(),
            description: "Easy jog".to_string(),
            distance: 5.0,
            moving_time: "00:25:00".to_string(),
            elapsed_time: "00:27:00".to_string(),
            total_elevation_gain: Some(50.0),
            activity_type: "Run".to_string(),
            sport_type: "Run".to_string(),
            start_date_local: "01 January 2026 07:00:00".to_string(),
            achievement_count: Some(2),
            average_speed: Some(3.33),
            max_speed: Some(4.5),
            average_watts: None,
            kilojoules: None,
            average_heartrate: Some(145.0),
            max_heartrate: Some(165.0),
            elev_high: Some(100.0),
            elev_low: Some(50.0),
            pr_count: Some(1),
        }
    }

    #[test]
    fn renders_with_activity() {
        let template = ActivityDetailsTemplate {
            activity: Some(test_activity()),
            prompt: "Generate a title".to_string(),
            map_points: vec![vec![1.0, 2.0]],
            heartrate_stream: vec![120.0, 130.0],
            altitude_stream: vec![100.0],
            velocity_stream: vec![],
        };
        let html = template.render().unwrap();
        assert!(html.contains("Back to Activities"));
        assert!(html.contains("Morning Run"));
        assert!(html.contains("AI Narrative Engine"));
        assert!(html.contains("Generate a title"));
    }

    #[test]
    fn renders_error_page_when_none() {
        let template = ActivityDetailsTemplate {
            activity: None,
            prompt: String::new(),
            map_points: vec![],
            heartrate_stream: vec![],
            altitude_stream: vec![],
            velocity_stream: vec![],
        };
        let html = template.render().unwrap();
        assert!(html.contains("Activity Not Found"));
        assert!(html.contains("Back to Activities"));
        assert!(!html.contains("AI Narrative Engine"));
    }

    #[test]
    fn renders_map_layer_dropdown() {
        let template = ActivityDetailsTemplate {
            activity: Some(test_activity()),
            prompt: String::new(),
            map_points: vec![],
            heartrate_stream: vec![],
            altitude_stream: vec![],
            velocity_stream: vec![],
        };
        let html = template.render().unwrap();
        assert!(html.contains("Heart Rate"));
        assert!(html.contains("Elevation"));
        assert!(html.contains("Speed"));
        assert!(html.contains("Route"));
    }

    #[test]
    fn renders_stream_data_as_json() {
        let template = ActivityDetailsTemplate {
            activity: Some(test_activity()),
            prompt: String::new(),
            map_points: vec![vec![1.5, 2.5]],
            heartrate_stream: vec![140.0, 150.0],
            altitude_stream: vec![],
            velocity_stream: vec![3.5],
        };
        let html = template.render().unwrap();
        assert!(html.contains("[140.0,150.0]"));
        assert!(html.contains("[3.5]"));
    }
}
