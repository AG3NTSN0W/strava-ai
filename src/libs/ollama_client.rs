use crate::libs::StravAIError;
use crate::libs::models::ollama::{OllamaRequest, OllamaResponse, Response};
use crate::libs::models::strava::activity::Activity;
use log::{debug, error};
use std::env;
use std::error::Error;

pub struct OllamaClient {}

impl OllamaClient {
    fn get_url() -> Option<String> {
        match env::var("OLLAMA_URL")
            .ok()
            .map(|id| id.trim().trim_matches('"').to_string())
        {
            Some(url) => Some(url),
            None => {
                error!("Missing OLLAMA_URL environment variable");
                None
            }
        }
    }

    /// Calls the Ollama API to generate title and description for an activity
    pub async fn generate_summary(
        request: OllamaRequest,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = match Self::get_url() {
            Some(url) => url,
            None => {
                return Err(Box::new(StravAIError(
                    "Missing OLLAMA_URL environment variable".into(),
                )));
            }
        };
        debug!("URL: {url}");

        let response = client.post(url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Ollama API error: {status} - {error_text}");
            return Err(format!("Ollama API returned status {status}: {error_text}").into());
        }

        let ollama_response: OllamaResponse = response.json().await?;
        debug!("Response: {ollama_response:?}");
        Ok(serde_json::from_str(&ollama_response.response)?)
    }

    pub async fn generate_activity_summary(
        activity: &Activity,
        prompt: &str,
    ) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let request: OllamaRequest = OllamaRequest {
            model: "llama3.2".to_string(),
            prompt: format!("The Data: {}", activity.to_json_string()),
            system: prompt.to_string(),
            stream: false,
            format: Default::default(),
        };
        Self::generate_summary(request).await
    }
}
