use super::{HtmlTemplate, format_date};
use crate::AppState;
use crate::libs::models::AthleteActivity;
use crate::libs::repository::ActivityRepository;
use askama::Template;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
pub struct ActivityFeedQueryParams {
    athlete_id: i64,
    sport_type: String,
}

#[derive(Template)]
#[template(path = "activity_feed_template.html")]
pub struct ActivityTemplate {
    activities: Vec<AthleteActivity>,
    sport_types: Vec<String>,
    sport_type: String,
}

impl ActivityTemplate {
    pub async fn new(
        State(state): State<Arc<AppState>>,
        query_params: ActivityFeedQueryParams,
    ) -> Self {
        let athlete_id = query_params.athlete_id;
        let activities = match query_params.sport_type.as_str() {
            "all" => ActivityRepository::get_by_athlete_id(&state.db_pools, athlete_id)
                .await
                .unwrap_or_else(|e| {
                    log::error!(
                        "Failed to fetch activities from database for athlete {athlete_id}: {e}"
                    );
                    vec![]
                }),
            _ => ActivityRepository::get_by_athlete_id_and_sport_type(
                &state.db_pools,
                athlete_id,
                &query_params.sport_type,
            )
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Failed to fetch activities from database for athlete {athlete_id}: {e}"
                );
                vec![]
            }),
        };

        let activities = activities
            .iter()
            .map(|activity| {
                let start_date_local = format_date(&activity.start_date_local);
                AthleteActivity {
                    start_date_local,
                    ..activity.clone()
                }
            })
            .collect();

        let sport_types =
            ActivityRepository::get_sport_types_by_athlete(&state.db_pools, athlete_id)
                .await
                .unwrap_or_else(|e| {
                    log::error!("Failed to fetch sport types from database: {e}");
                    vec![]
                });

        let sport_type: String = if query_params.sport_type.is_empty() {
            "all".to_string()
        } else {
            query_params.sport_type
        };

        Self {
            activities,
            sport_types,
            sport_type,
        }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    query_params: Query<ActivityFeedQueryParams>,
) -> impl IntoResponse {
    HtmlTemplate(ActivityTemplate::new(state, query_params.0).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use askama::Template;

    fn test_activity() -> AthleteActivity {
        AthleteActivity {
            id: 1,
            athlete_id: 1,
            name: "Morning Run".to_string(),
            description: "Easy".to_string(),
            distance: 5.0,
            moving_time: "00:25:00".to_string(),
            elapsed_time: "00:27:00".to_string(),
            total_elevation_gain: Some(50.0),
            activity_type: "Run".to_string(),
            sport_type: "Run".to_string(),
            start_date_local: "01 January 2026".to_string(),
            achievement_count: None,
            average_speed: None,
            max_speed: None,
            average_watts: None,
            kilojoules: None,
            average_heartrate: None,
            max_heartrate: None,
            elev_high: None,
            elev_low: None,
            pr_count: None,
        }
    }

    #[test]
    fn renders_with_activities() {
        let template = ActivityTemplate {
            activities: vec![test_activity()],
            sport_types: vec!["Run".to_string(), "Ride".to_string()],
            sport_type: "all".to_string(),
        };
        let html = template.render().unwrap();
        assert!(html.contains("Morning Run"));
        assert!(html.contains("5")); // distance
    }

    #[test]
    fn renders_empty_activities() {
        let template = ActivityTemplate {
            activities: vec![],
            sport_types: vec![],
            sport_type: "all".to_string(),
        };
        let html = template.render().unwrap();
        assert!(html.is_empty() == false);
    }
}
