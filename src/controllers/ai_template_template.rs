use super::HtmlTemplate;
use crate::AppState;
use crate::libs::models::ollama::Response;
use crate::libs::ollama_client::OllamaClient;
use crate::libs::repository::ActivityRepository;
use askama::Template;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use log::error;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct AiTemplateRequest {
    #[serde(deserialize_with = "deserialize_from_str")]
    activity_id: i64,
    prompt: String,
}

fn deserialize_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<T>().map_err(serde::de::Error::custom)
}

#[derive(Template)]
#[template(path = "ai_response_template.html")]
pub struct AiResponseTemplate {
    athlete_id: i64,
    activity_id: i64,
    title: String,
    description: String,
}

impl From<(Response, i64, i64)> for AiResponseTemplate {
    fn from(r: (Response, i64, i64)) -> Self {
        AiResponseTemplate {
            athlete_id: r.1,
            activity_id: r.2,
            title: r.0.title,
            description: r.0.description,
        }
    }
}

impl AiResponseTemplate {
    pub async fn new(State(state): State<Arc<AppState>>, body: AiTemplateRequest) -> Self {
        let activity = ActivityRepository::get_by_id(&state.db_pools, body.activity_id)
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Failed to fetch activity from database for ID: [{}]. Error: {e}",
                    body.activity_id
                );

                None
            });

        if let Some(activity) = activity {
            let response =
                OllamaClient::generate_activity_summary(&activity.clone().into(), &body.prompt)
                    .await
                    .unwrap_or_else(|e| {
                        error!("Failed to generate activity summary. Error: {e}");
                        Response {
                            title: "[ERROR]".to_string(),
                            description: "Unable to generate title and a description".to_string(),
                        }
                    });

            return AiResponseTemplate::from((response, activity.athlete_id, activity.id));
        }

        Self {
            athlete_id: 0,
            activity_id: 0,
            title: "[ERROR]".to_string(),
            description: "Unable to generate title and a description".to_string(),
        }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    Json(body): Json<AiTemplateRequest>,
) -> impl IntoResponse {
    HtmlTemplate(AiResponseTemplate::new(state, body).await)
}
