use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct UpdateActivity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commute: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trainer: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_from_home: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub activity_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sport_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn creates_default_update_activity_with_all_none_fields() {
        let update = UpdateActivity::default();

        assert_eq!(update.commute, None);
        assert_eq!(update.trainer, None);
        assert_eq!(update.hide_from_home, None);
        assert_eq!(update.description, None);
        assert_eq!(update.name, None);
        assert_eq!(update.activity_type, None);
        assert_eq!(update.sport_type, None);
        assert_eq!(update.gear_id, None);
    }

    #[test]
    fn serializes_with_all_boolean_fields_set() {
        let update = UpdateActivity {
            commute: Some(true),
            trainer: Some(false),
            hide_from_home: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_value(&update).unwrap();

        assert_eq!(json["commute"], true);
        assert_eq!(json["trainer"], false);
        assert_eq!(json["hide_from_home"], true);
    }

    #[test]
    fn serializes_with_all_string_fields_set() {
        let update = UpdateActivity {
            name: Some("Updated Ride".to_string()),
            description: Some("Great workout".to_string()),
            activity_type: Some("Ride".to_string()),
            sport_type: Some("MountainBike".to_string()),
            gear_id: Some("b12345".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&update).unwrap();

        assert_eq!(json["name"], "Updated Ride");
        assert_eq!(json["description"], "Great workout");
        assert_eq!(json["type"], "Ride");
        assert_eq!(json["sport_type"], "MountainBike");
        assert_eq!(json["gear_id"], "b12345");
    }

    #[test]
    fn skips_serializing_none_fields() {
        let update = UpdateActivity {
            name: Some("New Name".to_string()),
            commute: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_value(&update).unwrap();

        assert!(json.get("description").is_none());
        assert!(json.get("trainer").is_none());
        assert!(json.get("hide_from_home").is_none());
        assert!(json.get("activity_type").is_none());
        assert!(json.get("sport_type").is_none());
        assert!(json.get("gear_id").is_none());
    }

    #[test]
    fn serializes_empty_struct_to_empty_object() {
        let update = UpdateActivity::default();

        let json = serde_json::to_value(&update).unwrap();

        assert_eq!(json, json!({}));
    }

    #[test]
    fn deserializes_with_all_boolean_fields() {
        let json_str = r#"{"commute": true, "trainer": false, "hide_from_home": true}"#;

        let update: UpdateActivity = serde_json::from_str(json_str).unwrap();

        assert_eq!(update.commute, Some(true));
        assert_eq!(update.trainer, Some(false));
        assert_eq!(update.hide_from_home, Some(true));
    }

    #[test]
    fn deserializes_with_all_string_fields() {
        let json_str = r#"{"name": "Evening Run", "description": "Fast pace", "type": "Run", "sport_type": "TrailRun", "gear_id": "g98765"}"#;

        let update: UpdateActivity = serde_json::from_str(json_str).unwrap();

        assert_eq!(update.name, Some("Evening Run".to_string()));
        assert_eq!(update.description, Some("Fast pace".to_string()));
        assert_eq!(update.activity_type, Some("Run".to_string()));
        assert_eq!(update.sport_type, Some("TrailRun".to_string()));
        assert_eq!(update.gear_id, Some("g98765".to_string()));
    }

    #[test]
    fn deserializes_partial_fields() {
        let json_str = r#"{"name": "Partial Update", "commute": true}"#;

        let update: UpdateActivity = serde_json::from_str(json_str).unwrap();

        assert_eq!(update.name, Some("Partial Update".to_string()));
        assert_eq!(update.commute, Some(true));
        assert_eq!(update.trainer, None);
        assert_eq!(update.description, None);
    }

    #[test]
    fn deserializes_empty_object() {
        let json_str = r#"{}"#;

        let update: UpdateActivity = serde_json::from_str(json_str).unwrap();

        assert_eq!(update, UpdateActivity::default());
    }

    #[test]
    fn deserializes_ignores_unknown_fields() {
        let json_str = r#"{"name": "Test", "unknown_field": "ignored", "another_unknown": 123}"#;

        let update: UpdateActivity = serde_json::from_str(json_str).unwrap();

        assert_eq!(update.name, Some("Test".to_string()));
    }

    #[test]
    fn handles_empty_string_fields() {
        let update = UpdateActivity {
            name: Some(String::new()),
            description: Some(String::new()),
            ..Default::default()
        };

        let json = serde_json::to_value(&update).unwrap();

        assert_eq!(json["name"], "");
        assert_eq!(json["description"], "");
    }

    #[test]
    fn handles_special_characters_in_string_fields() {
        let update = UpdateActivity {
            name: Some("Ride @ Morning \"Test\" 🚴".to_string()),
            description: Some("Line1\nLine2\tTabbed".to_string()),
            gear_id: Some("id-with-special_chars.123".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&update).unwrap();

        assert_eq!(json["name"], "Ride @ Morning \"Test\" 🚴");
        assert_eq!(json["description"], "Line1\nLine2\tTabbed");
        assert_eq!(json["gear_id"], "id-with-special_chars.123");
    }

    #[test]
    fn serializes_boolean_false_values() {
        let update = UpdateActivity {
            commute: Some(false),
            trainer: Some(false),
            hide_from_home: Some(false),
            ..Default::default()
        };

        let json = serde_json::to_value(&update).unwrap();

        assert_eq!(json["commute"], false);
        assert_eq!(json["trainer"], false);
        assert_eq!(json["hide_from_home"], false);
    }

    #[test]
    fn renames_activity_type_field_to_type_in_json() {
        let json_str = r#"{"type": "Ride"}"#;

        let update: UpdateActivity = serde_json::from_str(json_str).unwrap();

        assert_eq!(update.activity_type, Some("Ride".to_string()));

        let serialized = serde_json::to_value(&update).unwrap();
        assert!(serialized.get("type").is_some());
        assert!(serialized.get("activity_type").is_none());
    }

    #[test]
    fn clears_gear_with_none_value() {
        let update = UpdateActivity {
            gear_id: Some("none".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&update).unwrap();

        assert_eq!(json["gear_id"], "none");
    }

    #[test]
    fn round_trips_full_update_through_serialization() {
        let original = UpdateActivity {
            commute: Some(true),
            trainer: Some(false),
            hide_from_home: Some(true),
            description: Some("Test description".to_string()),
            name: Some("Test Ride".to_string()),
            activity_type: Some("Ride".to_string()),
            sport_type: Some("MountainBike".to_string()),
            gear_id: Some("b123".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: UpdateActivity = serde_json::from_str(&json).unwrap();

        assert_eq!(original.commute, deserialized.commute);
        assert_eq!(original.trainer, deserialized.trainer);
        assert_eq!(original.hide_from_home, deserialized.hide_from_home);
        assert_eq!(original.description, deserialized.description);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.activity_type, deserialized.activity_type);
        assert_eq!(original.sport_type, deserialized.sport_type);
        assert_eq!(original.gear_id, deserialized.gear_id);
    }
}
