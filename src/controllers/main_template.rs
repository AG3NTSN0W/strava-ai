use super::HtmlTemplate;
use crate::AppState;
use crate::libs::models::athlete::AthleteDisplay;
use crate::libs::repository::AthleteRepository;
use askama::Template;
use axum::extract::State;
use axum::response::IntoResponse;
use std::sync::Arc;

#[derive(Template)]
#[template(path = "main_template.html")]
pub struct HomeTemplate {
    strava_client_id: i32,
    athletes: Vec<AthleteDisplay>,
}

impl HomeTemplate {
    pub async fn new(State(state): State<Arc<AppState>>) -> Self {
        let athletes = AthleteRepository::get_all(&state.db_pools)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to fetch athletes from database: {e}");
                vec![]
            })
            .into_iter()
            .map(AthleteDisplay::from)
            .collect();

        Self {
            strava_client_id: state.client_id,
            athletes,
        }
    }
}

pub async fn get_template(state: State<Arc<AppState>>) -> impl IntoResponse {
    HtmlTemplate(HomeTemplate::new(state).await)
}
