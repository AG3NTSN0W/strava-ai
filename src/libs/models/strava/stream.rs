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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserializes_from_strava_format() {
        let json_str = r#"{"type":"heartrate","data":[120,130,140],"series_type":"distance","original_size":3,"resolution":"high"}"#;
        let resp: StreamResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.stream_type, "heartrate");
        assert_eq!(resp.data, json!([120, 130, 140]));
        assert_eq!(resp.original_size, 3);
    }

    #[test]
    fn deserializes_latlng_stream() {
        let json_str = r#"{"type":"latlng","data":[[1.1,2.2],[3.3,4.4]],"series_type":"distance","original_size":2,"resolution":"high"}"#;
        let resp: StreamResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.stream_type, "latlng");
        assert_eq!(resp.data, json!([[1.1, 2.2], [3.3, 4.4]]));
    }

    #[test]
    fn serializes_with_type_rename() {
        let resp = StreamResponse {
            stream_type: "altitude".to_string(),
            data: json!([100.0, 105.0]),
            series_type: "distance".to_string(),
            original_size: 2,
            resolution: "high".to_string(),
        };
        let val = serde_json::to_value(&resp).unwrap();
        assert!(val.get("type").is_some());
        assert!(val.get("stream_type").is_none());
    }
}
