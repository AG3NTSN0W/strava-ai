use crate::libs::models::strava::activity::{Activity, ActivityMap};
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
    pub summary_polyline: String,
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
            map: ActivityMap {
                summary_polyline: a.summary_polyline,
            },
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
            summary_polyline: a.map.summary_polyline,
        }
    }
}
