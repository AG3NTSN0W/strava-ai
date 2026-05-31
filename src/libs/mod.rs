use thiserror::Error;

pub mod models;
pub mod repository;
pub mod strava_client;
pub mod ollama_client;
pub mod scheduler;
pub mod rate_limit;

pub static DEFAULT_PROMPT: &str = "Write a short, funny Strava title and a two-sentence description for an activity.\
         Use max heart rate to determine if activity was 'Easy,Brutal,Insane'.\
         Use a pun if possible, but keep it low-key and cool.";

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Error(String),

    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Failed to deserialize response: {0}")]
    ToResponse(#[from] serde_json::Error),
}
