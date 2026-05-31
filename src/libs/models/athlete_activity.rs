use crate::libs::models::strava::activity::Activity;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AthleteActivity {
    pub id: i64,
    pub athlete_id: i64,
    pub name: String,
    pub description: String,
    pub distance: f64,
    pub moving_time: String,
    pub elapsed_time: String,
    pub total_elevation_gain: Option<f64>,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub sport_type: String,
    pub start_date_local: String,
    pub achievement_count: Option<i32>,
    pub average_speed: Option<f64>,
    pub max_speed: Option<f64>,
    pub average_watts: Option<f64>,
    pub kilojoules: Option<f64>,
    pub average_heartrate: Option<f64>,
    pub max_heartrate: Option<f64>,
    pub elev_high: Option<f64>,
    pub elev_low: Option<f64>,
    pub pr_count: Option<i32>,
}

impl AthleteActivity {
    pub fn to_activity_empty_name_desc(self) -> Activity {
        let mut activity: Activity = self.into();
        activity.name = String::new();
        activity.description = String::new();
        activity
    }
}

impl From<AthleteActivity> for Activity {
    fn from(a: AthleteActivity) -> Self {
        Activity {
            id: a.id,
            name: a.name,
            distance: a.distance,
            moving_time: a.moving_time,
            description: a.description,
            elapsed_time: a.elapsed_time,
            total_elevation_gain: a.total_elevation_gain,
            activity_type: a.activity_type.clone(),
            sport_type: a.sport_type.clone(),
            start_date_local: a.start_date_local.clone(),
            achievement_count: a.achievement_count,
            average_speed: a.average_speed,
            max_speed: a.max_speed,
            average_watts: a.average_watts,
            kilojoules: a.kilojoules,
            average_heartrate: a.average_heartrate,
            max_heartrate: a.max_heartrate,
            elev_high: a.elev_high,
            elev_low: a.elev_low,
            pr_count: a.pr_count,
        }
    }
}

impl From<(Activity, i64)> for AthleteActivity {
    fn from((a, athlete_id): (Activity, i64)) -> Self {
        AthleteActivity {
            id: a.id,
            athlete_id,
            name: a.name,
            distance: a.distance,
            moving_time: a.moving_time,
            description: a.description,
            elapsed_time: a.elapsed_time,
            total_elevation_gain: a.total_elevation_gain,
            activity_type: a.activity_type.clone(),
            sport_type: a.sport_type.clone(),
            start_date_local: a.start_date_local.clone(),
            achievement_count: a.achievement_count,
            average_speed: a.average_speed,
            max_speed: a.max_speed,
            average_watts: a.average_watts,
            kilojoules: a.kilojoules,
            average_heartrate: a.average_heartrate,
            max_heartrate: a.max_heartrate,
            elev_high: a.elev_high,
            elev_low: a.elev_low,
            pr_count: a.pr_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_athlete_activity() -> AthleteActivity {
        AthleteActivity {
            id: 1,
            athlete_id: 99,
            name: "Morning Run".to_string(),
            description: "Easy jog".to_string(),
            distance: 5.0,
            moving_time: "00:25:00".to_string(),
            elapsed_time: "00:27:00".to_string(),
            total_elevation_gain: Some(50.0),
            activity_type: "Run".to_string(),
            sport_type: "Run".to_string(),
            start_date_local: "2026-01-01T07:00:00Z".to_string(),
            achievement_count: Some(2),
            average_speed: Some(3.33),
            max_speed: Some(4.5),
            average_watts: None,
            kilojoules: None,
            average_heartrate: Some(145.0),
            max_heartrate: Some(165.0),
            elev_high: Some(100.0),
            elev_low: Some(50.0),
            pr_count: Some(1),
        }
    }

    #[test]
    fn athlete_activity_to_activity_preserves_fields() {
        let aa = test_athlete_activity();
        let activity: Activity = aa.into();
        assert_eq!(activity.id, 1);
        assert_eq!(activity.name, "Morning Run");
        assert_eq!(activity.distance, 5.0);
        assert_eq!(activity.average_heartrate, Some(145.0));
    }

    #[test]
    fn activity_with_athlete_id_to_athlete_activity() {
        let activity = Activity {
            id: 2,
            name: "Ride".to_string(),
            description: "".to_string(),
            distance: 20.0,
            moving_time: "01:00:00".to_string(),
            elapsed_time: "01:05:00".to_string(),
            total_elevation_gain: Some(200.0),
            activity_type: "Ride".to_string(),
            sport_type: "Ride".to_string(),
            start_date_local: "2026-02-01T08:00:00Z".to_string(),
            achievement_count: None,
            average_speed: Some(5.5),
            max_speed: Some(12.0),
            average_watts: Some(150.0),
            kilojoules: Some(540.0),
            average_heartrate: None,
            max_heartrate: None,
            elev_high: None,
            elev_low: None,
            pr_count: None,
        };
        let aa: AthleteActivity = (activity, 77).into();
        assert_eq!(aa.id, 2);
        assert_eq!(aa.athlete_id, 77);
        assert_eq!(aa.average_watts, Some(150.0));
    }

    #[test]
    fn roundtrip_conversion_preserves_data() {
        let original = test_athlete_activity();
        let activity: Activity = original.clone().into();
        let back: AthleteActivity = (activity, original.athlete_id).into();
        assert_eq!(back.id, original.id);
        assert_eq!(back.athlete_id, original.athlete_id);
        assert_eq!(back.name, original.name);
        assert_eq!(back.distance, original.distance);
        assert_eq!(back.average_heartrate, original.average_heartrate);
    }
}
