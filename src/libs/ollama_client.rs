use crate::libs::models::ollama::{OllamaRequest, OllamaResponse, Response};
use crate::libs::models::strava::activity::Activity;
use log::{debug, error};
use std::env;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OllamaClientError {
    #[error("Ollama API error {context}: HTTP status {status_code}")]
    UnknowError {
        context: String,
        status_code: StatusCode,
    },

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Failed to deserialize response: {0}")]
    ToResponse(#[from] serde_json::Error),

    #[error("Missing OLLAMA_URL environment variable")]
    UrlNotFound(),
}

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
    ) -> Result<Response, OllamaClientError> {
        let client = reqwest::Client::new();

        let url = match Self::get_url() {
            Some(url) => url,
            None => {
                return Err(OllamaClientError::UrlNotFound());
            }
        };
        debug!("URL: {url}");

        let response = client.post(url).json(&request).send().await?;

        if !response.status().is_success() {
            let status_code = response.status();
            let context = response.text().await.unwrap_or_default();
            error!("Ollama API error: {status_code} - {context}");
            return Err(OllamaClientError::UnknowError{
                context,
                status_code,
            });
        }

        let ollama_response: OllamaResponse = response.json().await?;
        debug!("Response: {ollama_response:?}");
        Ok(serde_json::from_str(&ollama_response.response)?)
    }

    pub async fn generate_activity_summary(
        activity: &Activity,
        prompt: &str,
    ) -> Result<Response, OllamaClientError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_error_display() {
        let err = OllamaClientError::UnknowError {
            context: "model not found".to_string(),
            status_code: StatusCode::NOT_FOUND,
        };
        assert_eq!(err.to_string(), "Ollama API error model not found: HTTP status 404 Not Found");
    }

    #[test]
    fn url_not_found_display() {
        let err = OllamaClientError::UrlNotFound();
        assert_eq!(err.to_string(), "Missing OLLAMA_URL environment variable");
    }
}
