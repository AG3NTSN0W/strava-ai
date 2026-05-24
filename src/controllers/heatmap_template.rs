use super::HtmlTemplate;
use crate::AppState;
use crate::libs::repository::activity_stream_repository::{
    ActivityDataResults, ActivityStreamResults, AltitudeLatLngResults, HeartRateLatLngResults,
    VelocityLatLngResults,
};
use crate::libs::repository::{ActivityRepository, ActivityStreamRepository};
use askama::Template;
use axum::extract::{RawQuery, State};
use axum::response::IntoResponse;
use chrono::{Local, Months};
use log::debug;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
pub struct ActivitiesData {
    activity_count: i32,
    avg_distance: f32,
    total_distance: f32,
    avg_time: String,
    total_time: String,
    avg_elevation: f32,
    average_heartrate: i32,
}

#[derive(Debug, Clone, Serialize)]
struct HeatMapData {
    points: Vec<Vec<Vec<f64>>>,
    max_count: i32,
    low_count: i32,
    palette: HashMap<String, String>,
    frequency_colors: String,
    activities_data: ActivitiesData,
}

struct HeatmapQueryParams {
    athlete_id: i64,
    sport_types: Vec<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    map_type: String,
}

impl HeatmapQueryParams {
    fn from_raw(query: &str) -> Option<Self> {
        let mut athlete_id = None;
        let mut sport_types = Vec::new();
        let mut date_from = None;
        let mut date_to = None;
        let mut map_type = String::from("frequency");

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            let val = urlencoding::decode(val).unwrap_or_default().into_owned();
            match key {
                "athlete_id" => athlete_id = val.parse().ok(),
                "sport_type" if !val.is_empty() => sport_types.push(val),
                "date_from" if !val.is_empty() => date_from = Some(val),
                "date_to" if !val.is_empty() => date_to = Some(val),
                "map_type" if !val.is_empty() => map_type = val,
                _ => {}
            }
        }
        Some(Self {
            athlete_id: athlete_id?,
            sport_types,
            date_from,
            date_to,
            map_type,
        })
    }
}

#[derive(Template)]
#[template(path = "heatmap_template.html")]
pub struct HeatmapTemplate {
    athlete_id: i64,
    sport_types: Vec<String>,
    selected_sports: Vec<String>,
    date_from: String,
    date_to: String,
    heat_map_data: HeatMapData,
    map_type: String,
}

impl HeatmapTemplate {
    async fn new(state: &Arc<AppState>, params: HeatmapQueryParams) -> Self {
        let athlete_id = params.athlete_id;
        let selected_sports = params.sport_types;
        let map_type = params.map_type;
        let today = Local::now().date_naive();
        let one_month_ago = today.checked_sub_months(Months::new(1)).unwrap_or(today);
        let date_from = params
            .date_from
            .unwrap_or_else(|| one_month_ago.to_string());
        let date_to = params.date_to.unwrap_or_else(|| today.to_string());

        let sport_types =
            ActivityRepository::get_sport_types_by_athlete(&state.db_pools, athlete_id)
                .await
                .unwrap_or_default();

        let sport_refs: Vec<&str> = selected_sports.iter().map(|s| s.as_str()).collect();
        let sport_filter = if sport_refs.is_empty() {
            None
        } else {
            Some(sport_refs.as_slice())
        };
        let from_filter = if date_from.is_empty() {
            None
        } else {
            Some(date_from.as_str())
        };
        let to_filter = if date_to.is_empty() {
            None
        } else {
            Some(date_to.as_str())
        };

        let heat_map_data = match map_type.as_str() {
            "heartrate" => {
                build_heart_rate_heat_map(state, athlete_id, sport_filter, from_filter, to_filter)
                    .await
            }
            "altitude" => {
                build_altitude_heat_map(state, athlete_id, sport_filter, from_filter, to_filter)
                    .await
            }
            "velocity" => {
                build_velocity_heat_map(state, athlete_id, sport_filter, from_filter, to_filter)
                    .await
            }
            _ => {
                build_frequency_heat_map(state, athlete_id, sport_filter, from_filter, to_filter)
                    .await
            }
        };

        debug!("Routes found and heatmap built: {heat_map_data:?}");

        Self {
            athlete_id,
            sport_types,
            selected_sports,
            date_from,
            date_to,
            heat_map_data,
            map_type,
        }
    }
}

pub async fn get_template(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    match HeatmapQueryParams::from_raw(query.as_deref().unwrap_or("")) {
        Some(p) => HtmlTemplate(HeatmapTemplate::new(&state, p).await).into_response(),
        None => axum::http::StatusCode::BAD_REQUEST.into_response(),
    }
}

// --- Heat map builders ---

async fn build_frequency_heat_map(
    state: &AppState,
    athlete_id: i64,
    sport_filter: Option<&[&str]>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> HeatMapData {
    let routes: ActivityStreamResults<Vec<Vec<Vec<f64>>>> =
        ActivityStreamRepository::get_latlng_points_filtered(
            &state.db_pools,
            athlete_id,
            sport_filter,
            date_from,
            date_to,
        )
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to fetch latlng streams: {e}");
            ActivityStreamResults {
                data: vec![],
                activity_data: vec![],
            }
        });

    // Grid resolution ~100m at equator
    let precision = 1000.0;
    let routes_activity_data = routes.activity_data;

    // Count distinct routes per grid cell
    let mut cell_counts: HashMap<(i64, i64), u32> = HashMap::new();
    for route in &routes.data {
        let mut visited = HashSet::new();
        for point in route {
            if point.len() >= 2 {
                let cell = ((point[0] * precision) as i64, (point[1] * precision) as i64);
                if visited.insert(cell) {
                    *cell_counts.entry(cell).or_insert(0) += 1;
                }
            }
        }
    }

    let points: Vec<Vec<Vec<f64>>> = routes
        .data
        .into_iter()
        .map(|route| {
            route
                .into_iter()
                .filter_map(|point| {
                    if point.len() >= 2 {
                        let cell = ((point[0] * precision) as i64, (point[1] * precision) as i64);
                        let count = *cell_counts.get(&cell).unwrap_or(&1);
                        Some(vec![point[0], point[1], count as f64])
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();

    let max_count = points.len() as i32;
    let low_count = 0;

    let data = build_activities_data(routes_activity_data);

    HeatMapData {
        max_count,
        low_count,
        points,
        frequency_colors:
            "hsl(240,100%,50%),hsl(180,100%,50%),hsl(120,100%,50%),hsl(60,100%,50%),hsl(0,100%,50%)"
                .to_string(),
        palette: HashMap::from([
            ("0.0".into(), "hsl(240,100%,50%)".into()),
            ("0.2".into(), "hsl(180,100%,50%)".into()),
            ("0.5".into(), "hsl(120,100%,50%)".into()),
            ("0.7".into(), "hsl(60,100%,50%)".into()),
            ("1.0".into(), "hsl(0,100%,50%)".into()),
        ]),
        activities_data: data,
    }
}

/// Shared logic for metric-based heat maps (heartrate, altitude, velocity).
/// Averages metric values per grid cell, then maps each point to [lat, lng, avg_metric].
fn build_metric_heat_map(
    latlng: Vec<Vec<Vec<f64>>>,
    metrics: &[Vec<f64>],
    activity_data: Vec<ActivityDataResults>,
    precision: f64,
    skip_zero: bool,
    frequency_colors: &str,
    palette: HashMap<String, String>,
) -> HeatMapData {
    // Accumulate metric values per grid cell
    let mut cell_sums: HashMap<(i64, i64), (f64, u32)> = HashMap::new();
    for (route, metric_stream) in latlng.iter().zip(metrics.iter()) {
        for (point, &val) in route.iter().zip(metric_stream.iter()) {
            if point.len() >= 2 && (!skip_zero || val > 0.0) {
                let cell = ((point[0] * precision) as i64, (point[1] * precision) as i64);
                let entry = cell_sums.entry(cell).or_insert((0.0, 0));
                entry.0 += val;
                entry.1 += 1;
            }
        }
    }

    // Pre-compute averages
    let cell_avgs: HashMap<(i64, i64), f64> = cell_sums
        .into_iter()
        .map(|(cell, (sum, count))| (cell, sum / count as f64))
        .collect();

    let mut max_val: f64 = 0.0;
    let mut min_val: f64 = f64::MAX;

    let points: Vec<Vec<Vec<f64>>> = latlng
        .into_iter()
        .map(|route| {
            route
                .into_iter()
                .filter_map(|point| {
                    if point.len() < 2 {
                        return None;
                    }
                    let cell = ((point[0] * precision) as i64, (point[1] * precision) as i64);
                    let avg = *cell_avgs.get(&cell).unwrap_or(&0.0);
                    if avg > 0.0 {
                        max_val = max_val.max(avg);
                        min_val = min_val.min(avg);
                    }
                    Some(vec![point[0], point[1], avg])
                })
                .collect()
        })
        .collect();

    HeatMapData {
        max_count: max_val as i32,
        low_count: if min_val == f64::MAX {
            0
        } else {
            min_val as i32
        },
        points,
        frequency_colors: frequency_colors.to_string(),
        palette,
        activities_data: build_activities_data(activity_data),
    }
}

async fn build_heart_rate_heat_map(
    state: &AppState,
    athlete_id: i64,
    sport_filter: Option<&[&str]>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> HeatMapData {
    let results = ActivityStreamRepository::get_latlng_heart_rate_points_filtered(
        &state.db_pools,
        athlete_id,
        sport_filter,
        date_from,
        date_to,
    )
    .await
    .unwrap_or_else(|e| {
        log::error!("Failed to fetch latlng streams: {e}");

        ActivityStreamResults {
            data: HeartRateLatLngResults {
                heartrate: vec![],
                latlng: vec![],
            },
            activity_data: vec![],
        }
    });

    build_metric_heat_map(
        results.data.latlng,
        &results.data.heartrate,
        results.activity_data,
        1000.0,
        true,
        "#ffffff,#ffe0e0,#ffb3b3,#ff6666,#cc0000",
        HashMap::from([
            ("0.0".into(), "#ffffff".into()),
            ("0.25".into(), "#ffe0e0".into()),
            ("0.5".into(), "#ffb3b3".into()),
            ("0.75".into(), "#ff6666".into()),
            ("1.0".into(), "#cc0000".into()),
        ]),
    )
}

async fn build_altitude_heat_map(
    state: &AppState,
    athlete_id: i64,
    sport_filter: Option<&[&str]>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> HeatMapData {
    let results = ActivityStreamRepository::get_latlng_altitude_points_filtered(
        &state.db_pools,
        athlete_id,
        sport_filter,
        date_from,
        date_to,
    )
    .await
    .unwrap_or_else(|e| {
        log::error!("Failed to fetch latlng streams: {e}");
        ActivityStreamResults {
            data: AltitudeLatLngResults {
                altitude: vec![],
                latlng: vec![],
            },
            activity_data: vec![],
        }
    });

    build_metric_heat_map(
        results.data.latlng,
        &results.data.altitude,
        results.activity_data,
        1000.0,
        false,
        "#006400,#228B22,#ADFF2F,#FFD700,#8B4513",
        HashMap::from([
            ("0.0".into(), "#006400".into()),
            ("0.25".into(), "#228B22".into()),
            ("0.5".into(), "#ADFF2F".into()),
            ("0.75".into(), "#FFD700".into()),
            ("1.0".into(), "#8B4513".into()),
        ]),
    )
}

async fn build_velocity_heat_map(
    state: &AppState,
    athlete_id: i64,
    sport_filter: Option<&[&str]>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> HeatMapData {
    let results = ActivityStreamRepository::get_latlng_velocity_points_filtered(
        &state.db_pools,
        athlete_id,
        sport_filter,
        date_from,
        date_to,
    )
    .await
    .unwrap_or_else(|e| {
        log::error!("Failed to fetch latlng streams: {e}");
        ActivityStreamResults {
            data: VelocityLatLngResults {
                velocity: vec![],
                latlng: vec![],
            },
            activity_data: vec![],
        }
    });

    build_metric_heat_map(
        results.data.latlng,
        &results.data.velocity,
        results.activity_data,
        1000.0,
        true,
        "#0000FF,#00BFFF,#00FF00,#FFA500,#FF0000",
        HashMap::from([
            ("0.0".into(), "#0000FF".into()),
            ("0.25".into(), "#00BFFF".into()),
            ("0.5".into(), "#00FF00".into()),
            ("0.75".into(), "#FFA500".into()),
            ("1.0".into(), "#FF0000".into()),
        ]),
    )
}

fn build_activities_data(activity_data: Vec<ActivityDataResults>) -> ActivitiesData {
    let count = activity_data.len();
    if count == 0 {
        return ActivitiesData {
            activity_count: 0,
            avg_distance: 0.0,
            total_distance: 0.0,
            avg_time: "00:00:00".to_string(),
            total_time: "00:00:00".to_string(),
            avg_elevation: 0.0,
            average_heartrate: 0,
        };
    }

    let (total_distance, total_seconds, total_elevation, total_heartrate, hr_count) =
        activity_data.iter().fold(
            (0.0f32, 0u64, 0.0f32, 0i64, 0u32),
            |(dist, secs, elev, hr, hrc), a| {
                let (hr_add, hrc_add) = if a.average_heartrate > 0 {
                    (a.average_heartrate as i64, 1)
                } else {
                    (0, 0)
                };
                (
                    dist + a.distance,
                    secs + parse_hms_to_seconds(&a.elapsed_time),
                    elev + a.total_elevation_gain,
                    hr + hr_add,
                    hrc + hrc_add,
                )
            },
        );

    let cf = count as f32;
    ActivitiesData {
        activity_count: count as i32,
        avg_distance: (total_distance / cf).round(),
        total_distance: total_distance.round(),
        avg_time: seconds_to_hms(total_seconds / count as u64),
        total_time: seconds_to_hms(total_seconds),
        avg_elevation: (total_elevation / cf).round(),
        average_heartrate: if hr_count > 0 {
            (total_heartrate / hr_count as i64) as i32
        } else {
            0
        },
    }
}

fn parse_hms_to_seconds(hms: &str) -> u64 {
    let parts: Vec<&str> = hms.split(':').collect();
    if parts.len() == 3 {
        let h: u64 = parts[0].parse().unwrap_or(0);
        let m: u64 = parts[1].parse().unwrap_or(0);
        let s: u64 = parts[2].parse().unwrap_or(0);
        h * 3600 + m * 60 + s
    } else {
        0
    }
}

fn seconds_to_hms(total: u64) -> String {
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
