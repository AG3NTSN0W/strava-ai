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

        if let Some(athlete) = athlete {
            return Self {
                activity: Some(activity),
                prompt: athlete.prompt,
                map_points,
            };
        }

        Self {
            activity: Some(activity),
            prompt: DEFAULT_PROMPT.to_string(),
            map_points,
        }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    query_params: Query<TemplateQueryParams>,
) -> impl IntoResponse {
    HtmlTemplate(ActivityDetailsTemplate::new(state, query_params.0).await)
}
