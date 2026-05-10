use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Athlete {
    pub id: i64,
    pub firstname: String,
    pub lastname: String,
    pub premium: bool,
    pub refresh_token: String,
    pub auto_update: bool,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteDisplay {
    pub id: i64,
    pub firstname: String,
    pub lastname: String,
}

impl From<Athlete> for AthleteDisplay {
    fn from(a: Athlete) -> Self {
        AthleteDisplay {
            id: a.id,
            firstname: a.firstname,
            lastname: a.lastname,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_athlete() -> Athlete {
        Athlete {
            id: 42,
            firstname: "Jane".to_string(),
            lastname: "Smith".to_string(),
            premium: true,
            refresh_token: "token123".to_string(),
            auto_update: true,
            prompt: "Generate a title".to_string(),
        }
    }

    #[test]
    fn athlete_to_display_preserves_fields() {
        let athlete = test_athlete();
        let display: AthleteDisplay = athlete.into();
        assert_eq!(display.id, 42);
        assert_eq!(display.firstname, "Jane");
        assert_eq!(display.lastname, "Smith");
    }

    #[test]
    fn athlete_display_does_not_expose_sensitive_fields() {
        let athlete = test_athlete();
        let display: AthleteDisplay = athlete.into();
        let json = serde_json::to_value(&display).unwrap();
        assert!(json.get("refresh_token").is_none());
        assert!(json.get("prompt").is_none());
    }
}
