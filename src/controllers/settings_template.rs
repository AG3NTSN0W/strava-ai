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

#[cfg(test)]
mod tests {
    use super::*;
    use askama::Template;

    #[test]
    fn renders_with_settings() {
        let template = SettingsTemplate {
            prompt: "Write a funny title".to_string(),
            auto_update: true,
        };
        let html = template.render().unwrap();
        assert!(html.contains("Write a funny title"));
        assert!(html.contains(" checked "));
    }

    #[test]
    fn renders_with_auto_update_off() {
        let template = SettingsTemplate {
            prompt: "my prompt".to_string(),
            auto_update: false,
        };
        let html = template.render().unwrap();
        assert!(html.contains("my prompt"));
        assert!(!html.contains(" checked \n"));
    }
}
