use serde::{Deserialize, Serialize};

/// Represents a property schema in the format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    #[serde(rename = "type")]
    pub schema_type: String,
}

/// Represents the properties object in the format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatProperties {
    pub title: PropertySchema,
    pub description: PropertySchema,
}

/// Represents the format specification for the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Format {
    #[serde(rename = "type")]
    pub format_type: String,
    pub properties: FormatProperties,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            format_type: "object".to_string(),
            properties: FormatProperties {
                title: PropertySchema {
                    schema_type: "string".to_string(),
                },
                description: PropertySchema {
                    schema_type: "string".to_string(),
                },
            },
        }
    }
}

/// Represents a request to the Ollama API for generating activity title and description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub system: String,
    pub stream: bool,
    pub format: Format,
}

/// Represents the response from Ollama containing generated title and description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub done_reason: String,
}

/// Represents the response from Ollama containing generated title and description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub title: String,
    pub description: String,
}
