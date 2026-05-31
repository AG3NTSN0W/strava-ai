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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_and_deserializes() {
        let stream = ActivityStream {
            id: 1,
            activity_id: 100,
            stream_type: "heartrate".to_string(),
            data: "[120,130,140]".to_string(),
            series_type: "distance".to_string(),
            original_size: 3,
            resolution: "high".to_string(),
        };
        let json = serde_json::to_string(&stream).unwrap();
        let back: ActivityStream = serde_json::from_str(&json).unwrap();
        assert_eq!(back.activity_id, 100);
        assert_eq!(back.stream_type, "heartrate");
        assert_eq!(back.original_size, 3);
    }

    #[test]
    fn clone_creates_independent_copy() {
        let stream = ActivityStream {
            id: 1,
            activity_id: 100,
            stream_type: "latlng".to_string(),
            data: "[[1.0,2.0]]".to_string(),
            series_type: "distance".to_string(),
            original_size: 1,
            resolution: "high".to_string(),
        };
        let cloned = stream.clone();
        assert_eq!(cloned.stream_type, "latlng");
        assert_eq!(cloned.data, "[[1.0,2.0]]");
    }
}
