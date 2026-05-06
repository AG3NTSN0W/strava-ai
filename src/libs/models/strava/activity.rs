use log::debug;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityMap {
     pub summary_polyline: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(deserialize_with = "deserialize_distance_to_km")]
    pub distance: f64,
    #[serde(deserialize_with = "deserialize_seconds_to_time")]
    pub moving_time: String,
    #[serde(deserialize_with = "deserialize_seconds_to_time")]
    pub elapsed_time: String,
    pub total_elevation_gain: Option<f64>,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub sport_type: String,
    pub start_date_local: String,
    pub achievement_count: Option<i32>,
    pub average_speed: Option<f64>,
    pub max_speed: Option<f64>,
    #[serde(default)]
    pub average_watts: Option<f64>,
    pub kilojoules: Option<f64>,
    #[serde(default)]
    pub average_heartrate: Option<f64>,
    #[serde(default)]
    pub max_heartrate: Option<f64>,
    pub elev_high: Option<f64>,
    pub elev_low: Option<f64>,
    pub pr_count: Option<i32>,
    pub map: ActivityMap
}

impl Activity {
    pub fn to_json_string(&self) -> String {
        let string = serde_json::to_string(self).unwrap_or_else(|_| String::from("{}"));
        debug!("Activity string: {string}");
        string
    }
}

impl fmt::Display for Activity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let avg_hr_str = self
            .average_heartrate
            .map(|hr| format!("{hr:.0}"))
            .unwrap_or_else(|| "N/A".to_string());
        let max_hr_str = self
            .max_heartrate
            .map(|hr| format!("{hr:.0}"))
            .unwrap_or_else(|| "N/A".to_string());
        let elev_str = self
            .total_elevation_gain
            .map(|e| format!("{}", e as i32))
            .unwrap_or_else(|| "N/A".to_string());
        let pr_count = self.pr_count.unwrap_or(0);
        let average_speed = self.average_speed.unwrap_or(0.0);
        let max_speed = self.max_speed.unwrap_or(0.0);
        write!(
            f,
            "Activity Type {} : {} | Distance: {} km | Time: {} | Elevation: {elev_str} m | Average Heartrate: {avg_hr_str} bpm | Max Heartrate {max_hr_str} bpm | Personal records {pr_count} | Average speed {average_speed} km/h | Max Speed {max_speed} km/h",
            self.activity_type, self.sport_type, self.distance, self.moving_time,
        )
    }
}

fn deserialize_distance_to_km<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let meters = f64::deserialize(deserializer)?;
    let km = meters / 1000.0;
    Ok((km * 100.0).trunc() / 100.0)
}

fn deserialize_seconds_to_time<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = i32::deserialize(deserializer)?;
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    Ok(format!("{hours:02}:{minutes:02}:{secs:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_activity_json() -> String {
        let obj = serde_json::json!({
            "id": 12345678987654321i64,
            "name": "Morning Ride",
            "description": "Great ride!",
            "distance": 28099,
            "moving_time": 4207,
            "elapsed_time": 4410,
            "total_elevation_gain": 977.0,
            "type": "Ride",
            "sport_type": "Ride",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 36,
            "average_speed": 4.805,
            "max_speed": 17.28,
            "average_watts": 85.6,
            "kilojoules": 1160.1,
            "average_heartrate": 153.8,
            "max_heartrate": 173.0,
            "elev_high": 236.6,
            "elev_low": 1.0,
            "pr_count": 27
        });
        obj.to_string()
    }

    #[test]
    fn deserializes_valid_activity_with_all_fields() {
        let json = valid_activity_json();
        let activity: Activity = serde_json::from_str(&json).unwrap();

        assert_eq!(activity.id, 12345678987654321);
        assert_eq!(activity.name, "Morning Ride");
        assert_eq!(activity.activity_type, "Ride");
        assert_eq!(activity.sport_type, "Ride");
        assert_eq!(activity.achievement_count, Some(36));
        assert_eq!(activity.pr_count, Some(27));
    }

    #[test]
    fn converts_distance_from_meters_to_kilometers() {
        let json = valid_activity_json();
        let activity: Activity = serde_json::from_str(&json).unwrap();

        assert_eq!(activity.distance, 28.09);
    }

    #[test]
    fn converts_distance_zero_to_zero() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 0,
            "moving_time": 0,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.distance, 0.0);
    }

    #[test]
    fn converts_distance_with_decimal_values() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 1500.5,
            "moving_time": 0,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.distance, 1.50);
    }

    #[test]
    fn converts_moving_time_from_seconds_to_hh_mm_ss() {
        let json = valid_activity_json();
        let activity: Activity = serde_json::from_str(&json).unwrap();

        assert_eq!(activity.moving_time, "01:10:07");
    }

    #[test]
    fn converts_elapsed_time_from_seconds_to_hh_mm_ss() {
        let json = valid_activity_json();
        let activity: Activity = serde_json::from_str(&json).unwrap();

        assert_eq!(activity.elapsed_time, "01:13:30");
    }

    #[test]
    fn formats_time_with_zero_seconds() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 0,
            "moving_time": 3600,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.moving_time, "01:00:00");
    }

    #[test]
    fn formats_time_with_only_seconds() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 0,
            "moving_time": 45,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.moving_time, "00:00:45");
    }

    #[test]
    fn formats_time_with_only_minutes() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 0,
            "moving_time": 600,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.moving_time, "00:10:00");
    }

    #[test]
    fn formats_time_zero_seconds() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 0,
            "moving_time": 0,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.moving_time, "00:00:00");
    }

    #[test]
    fn formats_time_with_all_components() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 0,
            "moving_time": 3661,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.moving_time, "01:01:01");
    }

    #[test]
    fn formats_time_with_single_digit_components() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 0,
            "moving_time": 125,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.moving_time, "00:02:05");
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = json!({
            "id": 1,
            "name": "Test",
            "description": "",
            "distance": 0,
            "moving_time": 0,
            "elapsed_time": 0,
            "total_elevation_gain": 0.0,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": 0,
            "average_speed": 0.0,
            "max_speed": 0.0,
            "average_watts": 0.0,
            "kilojoules": 0.0,
            "average_heartrate": 0.0,
            "max_heartrate": 0.0,
            "elev_high": 0.0,
            "elev_low": 0.0,
            "pr_count": 0,
            "unknown_field": "ignored",
            "another_unknown": 123
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.id, 1);
        assert_eq!(activity.name, "Test");
    }

    #[test]
    fn deserializes_real_strava_api_response_with_all_optional_fields() {
        let json = json!({
            "resource_state": 2,
            "athlete": {
                "id": 46406773,
                "resource_state": 1
            },
            "name": "Morning Ride",
            "distance": 20168.7,
            "moving_time": 4080,
            "elapsed_time": 4928,
            "total_elevation_gain": 277.0,
            "type": "Ride",
            "sport_type": "Ride",
            "workout_type": null,
            "id": 1234567890i64,
            "start_date": "2026-03-19T02:49:41Z",
            "start_date_local": "2026-03-19T04:49:41Z",
            "timezone": "(GMT+02:00) Africa/Johannesburg",
            "utc_offset": 7200.0,
            "achievement_count": 1,
            "kudos_count": 4,
            "comment_count": 0,
            "athlete_count": 6,
            "photo_count": 0,
            "average_speed": 4.943,
            "max_speed": 14.7,
            "average_watts": 79.5,
            "device_watts": false,
            "kilojoules": 324.3,
            "has_heartrate": true,
            "average_heartrate": 136.8,
            "max_heartrate": 168.0,
            "elev_high": 106.2,
            "elev_low": 1.4,
            "pr_count": 0
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.id, 1234567890);
        assert_eq!(activity.name, "Morning Ride");
        assert_eq!(activity.activity_type, "Ride");
        assert_eq!(activity.average_watts, Some(79.5));
        assert_eq!(activity.average_heartrate, Some(136.8));
        assert_eq!(activity.max_heartrate, Some(168.0));
    }

    #[test]
    fn deserializes_strava_response_with_null_optional_fields() {
        let json = json!({
            "id": 12345i64,
            "name": "Test Activity",
            "description": "",
            "distance": 5000,
            "moving_time": 1800,
            "elapsed_time": 2000,
            "total_elevation_gain": null,
            "type": "Run",
            "sport_type": "Run",
            "start_date_local": "2026-03-14T04:54:17Z",
            "achievement_count": null,
            "average_speed": null,
            "max_speed": null,
            "average_watts": null,
            "kilojoules": null,
            "average_heartrate": null,
            "max_heartrate": null,
            "elev_high": null,
            "elev_low": null,
            "pr_count": null
        })
        .to_string();

        let activity: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(activity.id, 12345);
        assert_eq!(activity.average_watts, None);
        assert_eq!(activity.average_heartrate, None);
        assert_eq!(activity.max_heartrate, None);
        assert_eq!(activity.total_elevation_gain, None);
        assert_eq!(activity.achievement_count, None);
        assert_eq!(activity.average_speed, None);
        assert_eq!(activity.max_speed, None);
        assert_eq!(activity.kilojoules, None);
        assert_eq!(activity.elev_high, None);
        assert_eq!(activity.elev_low, None);
        assert_eq!(activity.pr_count, None);
    }
}
