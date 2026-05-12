use super::{HtmlTemplate, TemplateQueryParams};
use crate::AppState;
use crate::libs::models::AthleteActivity;
use crate::libs::repository::ActivityRepository;
use askama::Template;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "activity_feed_template.html")]
pub struct ActivityTemplate {
    activities: Vec<AthleteActivity>,
}

impl ActivityTemplate {
    pub async fn new(
        State(state): State<Arc<AppState>>,
        query_params: TemplateQueryParams,
    ) -> Self {
        let athlete_id = query_params.athlete_id;
        let activities = ActivityRepository::get_by_athlete_id(&state.db_pools, athlete_id)
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Failed to fetch activities from database for athlete {}: {e}",
                    athlete_id
                );
                vec![]
            })
            .iter()
            .map(|activity| {
                let formatted = activity
                    .start_date_local
                    .parse::<DateTime<Utc>>()
                    .map(|dt| dt.format("%d %B %Y %H:%M:%S").to_string())
                    .unwrap_or_else(|_| activity.start_date_local.clone());

                AthleteActivity {
                    start_date_local: formatted,
                    ..activity.clone()
                }
            })
            .collect();

        Self { activities }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    query_params: Query<TemplateQueryParams>,
) -> impl IntoResponse {
    HtmlTemplate(ActivityTemplate::new(state, query_params.0).await)
}
