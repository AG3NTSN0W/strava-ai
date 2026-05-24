use crate::AppState;
use askama::Template;
use axum::routing::{post, put};
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use log::error;
use serde::Deserialize;
use std::sync::Arc;

mod activity_details_template;
mod activity_feed_template;
mod ai_template_template;
pub mod callback_controller;
mod heatmap_template;
pub mod main_template;
mod settings_template;

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),

            Err(err) => {
                error!("Failed to render template: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to render template. Error: {err}"),
                )
                    .into_response()
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct TemplateQueryParams {
    athlete_id: i64,
    activity_id: Option<i64>,
}

pub fn format_date(start_date_local: &str) -> String {
    start_date_local
        .parse::<DateTime<Utc>>()
        .map(|dt| dt.format("%d %B %Y %H:%M:%S").to_string())
        .unwrap_or_else(|_| start_date_local.to_string())
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/stravai") }))
        .route("/stravai", get(main_template::get_template))
        .route("/exchange_token", get(callback_controller::exchange_token))
        .route("/generate", put(ai_template_template::get_template))
        .route("/athlete", get(activity_feed_template::get_template))
        .route("/activity", get(activity_details_template::get_template))
        .route("/activities", get(main_template::get_athlete))
        .route(
            "/update/activity",
            post(callback_controller::update_activity),
        )
        .route(
            "/update/settings",
            post(callback_controller::update_settings),
        )
        .route(
            "/update/prompt",
            post(callback_controller::update_prompt),
        )
        .route(
            "/update/auto",
            post(callback_controller::update_auto_update),
        )
        .route(
            "/backfill/streams",
            get(callback_controller::backfill_streams),
        )
        .route("/heat/map", get(heatmap_template::get_template))
        .route("/settings", get(settings_template::get_template))
        .with_state(app_state)
}
