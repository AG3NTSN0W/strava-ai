use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Athlete {
    pub id: i64,
    pub firstname: String,
    pub lastname: String,
    pub premium: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub athlete: Athlete
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub access_token: String,
    pub refresh_token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_athlete() -> Athlete {
        Athlete {
            id: 1,
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            premium: false,
        }
    }

    #[test]
    fn deserialize_valid_token_json() {
        let json = r#"{"access_token":"a9b723","refresh_token":"b5c569","athlete":{"id":1,"firstname":"John","lastname":"Doe","premium":false,"refresh_token":"b5c569"}}"#;
        let token: Result<Token, _> = serde_json::from_str(json);

        assert!(token.is_ok());
        let token = token.unwrap();
        assert_eq!(token.access_token, "a9b723");
        assert_eq!(token.refresh_token, "b5c569");
    }

    #[test]
    fn serialize_token_to_json() {
        let token = Token {
            access_token: "a9b723".to_string(),
            refresh_token: "b5c569".to_string(),
            athlete: create_test_athlete(),
        };

        let json = serde_json::to_string(&token).unwrap();
        let deserialized: Token = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.access_token, token.access_token);
        assert_eq!(deserialized.refresh_token, token.refresh_token);
    }

    #[test]
    fn deserialize_with_empty_strings() {
        let json = r#"{"access_token":"","refresh_token":"","athlete":{"id":1,"firstname":"","lastname":"","premium":false,"refresh_token":""}}"#;
        let token: Token = serde_json::from_str(json).unwrap();

        assert_eq!(token.access_token, "");
        assert_eq!(token.refresh_token, "");
    }

    #[test]
    fn deserialize_with_long_tokens() {
        let long_access = "a".repeat(500);
        let long_refresh = "b".repeat(500);
        let json = format!(
            r#"{{"access_token":"{long_access}","refresh_token":"{long_refresh}","athlete":{{"id":1,"firstname":"John","lastname":"Doe","premium":false,"refresh_token":"b5c569"}}}}"#
        );
        let token: Token = serde_json::from_str(&json).unwrap();

        assert_eq!(token.access_token.len(), 500);
        assert_eq!(token.refresh_token.len(), 500);
    }

    #[test]
    fn deserialize_missing_access_token_fails() {
        let json = r#"{"refresh_token":"b5c569","athlete":{"id":1,"firstname":"John","lastname":"Doe","premium":false,"refresh_token":"b5c569"}}"#;
        let result: Result<Token, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_missing_refresh_token_fails() {
        let json = r#"{"access_token":"a9b723","athlete":{"id":1,"firstname":"John","lastname":"Doe","premium":false,"refresh_token":"b5c569"}}"#;
        let result: Result<Token, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_with_extra_fields() {
        let json = r#"{"access_token":"a9b723","refresh_token":"b5c569","expires_in":3600,"athlete":{"id":1,"firstname":"John","lastname":"Doe","premium":false,"refresh_token":"b5c569"}}"#;
        let token: Token = serde_json::from_str(json).unwrap();

        assert_eq!(token.access_token, "a9b723");
        assert_eq!(token.refresh_token, "b5c569");
    }

    #[test]
    fn clone_creates_independent_copy() {
        let token1 = Token {
            access_token: "a9b723".to_string(),
            refresh_token: "b5c569".to_string(),
            athlete: create_test_athlete(),
        };
        let token2 = token1.clone();

        assert_eq!(token1.access_token, token2.access_token);
        assert_eq!(token1.refresh_token, token2.refresh_token);
    }

    #[test]
    fn debug_format_includes_fields() {
        let token = Token {
            access_token: "a9b723".to_string(),
            refresh_token: "b5c569".to_string(),
            athlete: create_test_athlete(),
        };
        let debug_str = format!("{token:?}");

        assert!(debug_str.contains("access_token"));
        assert!(debug_str.contains("refresh_token"));
        assert!(debug_str.contains("a9b723"));
        assert!(debug_str.contains("b5c569"));
    }

    #[test]
    fn deserialize_with_special_characters() {
        let json = r#"{"access_token":"a9b7@#$%","refresh_token":"b5c569!&*()","athlete":{"id":1,"firstname":"John","lastname":"Doe","premium":false,"refresh_token":"b5c569"}}"#;
        let token: Token = serde_json::from_str(json).unwrap();

        assert_eq!(token.access_token, "a9b7@#$%");
        assert_eq!(token.refresh_token, "b5c569!&*()");
    }

    #[test]
    fn deserialize_with_unicode_characters() {
        let json = r#"{"access_token":"a9b723émoji🚀","refresh_token":"b5c569日本語","athlete":{"id":1,"firstname":"John","lastname":"Doe","premium":false,"refresh_token":"b5c569"}}"#;
        let token: Token = serde_json::from_str(json).unwrap();

        assert_eq!(token.access_token, "a9b723émoji🚀");
        assert_eq!(token.refresh_token, "b5c569日本語");
    }
}
