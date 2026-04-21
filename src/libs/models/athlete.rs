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
