use super::HtmlTemplate;
use crate::AppState;
use crate::libs::models::{Athlete, AthleteActivity};
use crate::libs::repository::{ActivityRepository, AthleteRepository};
use askama::Template;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct ActivityTemplateQueryParams {
    athlete_id: i64,
}

#[derive(Deserialize)]
pub struct AthleteDisplay {
    athlete_id: i64,
    athlete_name: String,
    prompt: String,
    auto_update: bool,
}

impl Default for AthleteDisplay {
    fn default() -> Self {
        Self {
            athlete_id: 0,
            athlete_name: "".to_string(),
            prompt: "".to_string(),
            auto_update: false,
        }
    }
}

#[derive(Template)]
#[template(path = "activity_template.html")]
pub struct ActivityTemplate {
    athlete: AthleteDisplay,
    activities: Vec<AthleteActivity>,
}

impl ActivityTemplate {
    pub async fn new(
        State(state): State<Arc<AppState>>,
        query_params: ActivityTemplateQueryParams,
    ) -> Self {
        let athlete = AthleteRepository::get_by_id(&state.db_pools, query_params.athlete_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to fetch athletes from database: {e}");
                None
            });

        match athlete {
            Some(athlete) => Self::get_client_activities(state, &athlete).await,
            None => Self::no_client(),
        }
    }

    fn no_client() -> Self {
        Self {
            athlete: AthleteDisplay::default(),
            activities: vec![],
        }
    }

    async fn get_client_activities(state: Arc<AppState>, athlete: &Athlete) -> Self {
        let activities = ActivityRepository::get_by_athlete_id(&state.db_pools, athlete.id)
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Failed to fetch activities from database for athlete {}: {e}",
                    athlete.id
                );
                vec![]
            })
            .iter()
            .map(|activity| {
                let datetime: DateTime<Utc> = activity
                    .start_date_local
                    .parse()
                    .expect("Invalid date format");
                let formatted = datetime.format("%d %B %Y %H:%M:%S").to_string();

                AthleteActivity {
                    start_date_local: formatted,
                    ..activity.clone()
                }
            })
            .collect();
        let athlete = AthleteDisplay {
            athlete_id: athlete.id,
            athlete_name: format!("{} {}", athlete.firstname, athlete.lastname),
            prompt: athlete.prompt.to_string(),
            auto_update: athlete.auto_update,
        };
        Self {
            athlete,
            activities,
        }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    query_params: Query<ActivityTemplateQueryParams>,
) -> impl IntoResponse {
    HtmlTemplate(ActivityTemplate::new(state, query_params.0).await)
}
