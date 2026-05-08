use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamResponse {
    #[serde(rename = "type")]
    pub stream_type: String,
    pub data: serde_json::Value,
    pub series_type: String,
    pub original_size: i32,
    pub resolution: String,
}
