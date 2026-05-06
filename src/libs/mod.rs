use std::error::Error;
use std::fmt;
use serde::Deserialize;

pub mod models;
pub mod repository;
pub mod strava_client;
pub mod ollama_client;
pub mod scheduler;
pub mod polyline;

pub static DEFAULT_PROMPT: &str = "Write a short, funny Strava title and a two-sentence description for an activity.\
         Use max heart rate to determine if activity was 'Easy,Brutal,Insane'.\
         Use a pun if possible, but keep it low-key and cool.";

#[derive(Deserialize, Debug)]
struct StravAIError(String);

impl fmt::Display for StravAIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Custom error occurred: {}", self.0)
    }
}

// Minimal implementation of the Error trait
impl Error for StravAIError {}
