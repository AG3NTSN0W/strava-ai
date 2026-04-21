use super::HtmlTemplate;
use crate::AppState;
use crate::libs::models::AthleteActivity;
use crate::libs::models::ollama::Response;
use crate::libs::ollama_client::OllamaClient;
use crate::libs::repository::ActivityRepository;
use askama::Template;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct AiTemplateRequest {
    #[serde(deserialize_with = "deserialize_from_str")]
    activity_id: i64,
    #[serde(deserialize_with = "deserialize_from_str")]
    athlete_id: i64,
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
#[template(path = "ai_template.html")]
pub struct AiTemplate {
    activity_id: u64,
    athlete_id: i64,
    activity: Option<AthleteActivity>,
}

impl AiTemplate {
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
            let ai_response =
                OllamaClient::generate_activity_summary(&activity.clone().into(), &body.prompt)
                    .await
                    .unwrap_or_else(|e| {
                        error!("Failed to generate activity summary. Error: {e}");
                        Response {
                            title: "Error".to_string(),
                            description: "Unable to generate description".to_string(),
                        }
                    });

            let datetime: DateTime<Utc> = activity
                .start_date_local
                .parse()
                .expect("Invalid date format");
            let formatted = datetime.format("%d %B %Y %H:%M:%S").to_string();

            let activity = Some(AthleteActivity {
                name: ai_response.title,
                description: ai_response.description,
                start_date_local: formatted,
                ..activity.clone()
            });

            return Self {
                activity_id: body.activity_id as u64,
                athlete_id: body.athlete_id,
                activity,
            };
        }

        Self {
            activity_id: 0,
            athlete_id: 0,
            activity: None,
        }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    Json(body): Json<AiTemplateRequest>,
) -> impl IntoResponse {
    HtmlTemplate(AiTemplate::new(state, body).await)
}
