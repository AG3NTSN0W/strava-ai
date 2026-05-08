use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ActivityStream {
    pub id: i64,
    pub activity_id: i64,
    pub stream_type: String,
    pub data: String,
    pub series_type: String,
    pub original_size: i32,
    pub resolution: String,
}
