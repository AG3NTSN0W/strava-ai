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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_default_has_correct_structure() {
        let format = Format::default();
        assert_eq!(format.format_type, "object");
        assert_eq!(format.properties.title.schema_type, "string");
        assert_eq!(format.properties.description.schema_type, "string");
    }

    #[test]
    fn ollama_request_serializes_correctly() {
        let req = OllamaRequest {
            model: "llama3.2".to_string(),
            prompt: "test prompt".to_string(),
            system: "system msg".to_string(),
            stream: false,
            format: Format::default(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "llama3.2");
        assert_eq!(json["stream"], false);
        assert_eq!(json["format"]["type"], "object");
    }

    #[test]
    fn ollama_response_deserializes() {
        let json = r#"{"model":"llama3.2","created_at":"2026-01-01T00:00:00Z","response":"{\"title\":\"Run\",\"description\":\"A run\"}","done":true,"done_reason":"stop"}"#;
        let resp: OllamaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.model, "llama3.2");
        assert!(resp.done);
        assert_eq!(resp.done_reason, "stop");
    }

    #[test]
    fn response_deserializes() {
        let json = r#"{"title":"Morning Run","description":"A great morning run"}"#;
        let resp: Response = serde_json::from_str(json).unwrap();
        assert_eq!(resp.title, "Morning Run");
        assert_eq!(resp.description, "A great morning run");
    }
}
