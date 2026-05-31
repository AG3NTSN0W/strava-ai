use super::{HtmlTemplate, TemplateQueryParams};
use crate::AppState;
use crate::libs::models::athlete::AthleteDisplay;
use crate::libs::repository::AthleteRepository;
use askama::Template;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use std::sync::Arc;
use log::debug;

#[derive(Template)]
#[template(path = "athletes_template.html")]
struct AthletesPage {
    strava_client_id: i32,
    athletes: Vec<AthleteDisplay>,
}

#[derive(Template)]
#[template(path = "landing_page_template.html")]
struct LandingPage {
    strava_client_id: i32,
}

#[derive(Template)]
#[template(path = "main_template.html")]
pub struct MainTemplate {
    strava_client_id: i32,
    athlete: AthleteDisplay,
}

pub async fn get_template(state: State<Arc<AppState>>) -> Response {
    let strava_client_id = state.client_id;
    let athletes: Vec<AthleteDisplay> = AthleteRepository::get_all(&state.db_pools)
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to fetch athletes from database: {e}");
            vec![]
        })
        .into_iter()
        .map(AthleteDisplay::from)
        .collect();

    if athletes.is_empty() {
        return HtmlTemplate(LandingPage { strava_client_id }).into_response();
    }

    if athletes.len() == 1 {
        return HtmlTemplate(MainTemplate {
            strava_client_id,
            athlete: athletes.first().unwrap().clone(),
        })
        .into_response();
    }

    HtmlTemplate(AthletesPage {
        strava_client_id,
        athletes,
    })
    .into_response()
}

pub async fn get_athlete(
    state: State<Arc<AppState>>,
    query_params: Query<TemplateQueryParams>,
) -> Response {
    debug!("Query params: {query_params:?}");
    let strava_client_id = state.client_id;
    let athletes: Option<AthleteDisplay> =
        AthleteRepository::get_by_id(&state.db_pools, query_params.athlete_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to fetch athletes from database: {e}");
                None
            })
            .map(AthleteDisplay::from);

    if let Some(athlete) = athletes {
        return HtmlTemplate(MainTemplate {
            strava_client_id,
            athlete,
        })
        .into_response();
    };
    debug!("Athlete not found: {query_params:?}. redirect to main page");
    Redirect::permanent("/stravai").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use askama::Template;

    #[test]
    fn renders_main_template() {
        let template = MainTemplate {
            strava_client_id: 12345,
            athlete: AthleteDisplay {
                id: 1,
                firstname: "Jane".to_string(),
                lastname: "Doe".to_string(),
            },
        };
        let html = template.render().unwrap();
        assert!(html.contains("Jane"));
        assert!(html.contains("12345"));
    }

    #[test]
    fn renders_landing_page() {
        let template = LandingPage {
            strava_client_id: 99999,
        };
        let html = template.render().unwrap();
        assert!(html.contains("99999"));
    }

    #[test]
    fn renders_athletes_page() {
        let template = AthletesPage {
            strava_client_id: 11111,
            athletes: vec![
                AthleteDisplay { id: 1, firstname: "Alice".to_string(), lastname: "A".to_string() },
                AthleteDisplay { id: 2, firstname: "Bob".to_string(), lastname: "B".to_string() },
            ],
        };
        let html = template.render().unwrap();
        assert!(html.contains("Alice"));
        assert!(html.contains("Bob"));
    }
}
