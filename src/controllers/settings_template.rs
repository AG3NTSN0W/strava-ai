use super::{HtmlTemplate, TemplateQueryParams};
use crate::AppState;
use crate::libs::repository::AthleteRepository;
use askama::Template;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use std::sync::Arc;

#[derive(Template)]
#[template(path = "settings_template.html")]
pub struct SettingsTemplate {
    prompt: String,
    auto_update: bool,
}

impl SettingsTemplate {
    pub async fn new(
        State(state): State<Arc<AppState>>,
        query_params: TemplateQueryParams,
    ) -> Self {
        let athlete = AthleteRepository::get_by_id(&state.db_pools, query_params.athlete_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to fetch athletes from database: {e}");
                None
            });

        match athlete {
            None => Self {
                auto_update: false,
                prompt: "".to_string(),
            },
            Some(a) => Self {
                auto_update: a.auto_update,
                prompt: a.prompt,
            },
        }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    query_params: Query<TemplateQueryParams>,
) -> impl IntoResponse {
    HtmlTemplate(SettingsTemplate::new(state, query_params.0).await)
}
