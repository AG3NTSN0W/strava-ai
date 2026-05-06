use std::sync::Arc;
use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum::routing::{post, put};
use log::error;
use crate::AppState;

pub mod callback_controller;
pub mod main_template;
mod ai_template;
mod activity_template;
mod heatmap_template;

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

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/stravai") }))
        .route("/stravai", get(main_template::get_template))
        .route("/exchange_token", get(callback_controller::exchange_token))
        .route("/generate", put(ai_template::get_template))
        .route("/athlete", get(activity_template::get_template))
        .route("/update/activity", post(callback_controller::update_activity))
        .route("/update/settings", post(callback_controller::update_settings))
        .route("/heat/map", get(heatmap_template::get_template))
        .with_state(app_state)
}
