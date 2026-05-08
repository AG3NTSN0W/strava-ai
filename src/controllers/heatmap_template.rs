use super::HtmlTemplate;
use crate::AppState;
use crate::libs::repository::activity_stream_repository::HeartRateLatLngResults;
use crate::libs::repository::{ActivityRepository, ActivityStreamRepository};
use askama::Template;
use axum::extract::{RawQuery, State};
use axum::response::IntoResponse;
use chrono::{Local, Months};
use log::debug;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

struct HeatmapQueryParams {
    athlete_id: i64,
    sport_types: Vec<String>,
    date_from: Option<String>,
    date_to: Option<String>,
}

impl HeatmapQueryParams {
    fn from_raw(query: &str) -> Option<Self> {
        let mut athlete_id = None;
        let mut sport_types = Vec::new();
        let mut date_from = None;
        let mut date_to = None;

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            let val = urlencoding::decode(val).unwrap_or_default().into_owned();
            match key {
                "athlete_id" => athlete_id = val.parse().ok(),
                "sport_type" => {
                    if !val.is_empty() {
                        sport_types.push(val);
                    }
                }
                "date_from" => {
                    if !val.is_empty() {
                        date_from = Some(val);
                    }
                }
                "date_to" => {
                    if !val.is_empty() {
                        date_to = Some(val);
                    }
                }
                _ => {}
            }
        }
        Some(Self {
            athlete_id: athlete_id?,
            sport_types,
            date_from,
            date_to,
        })
    }
}

#[derive(Template)]
#[template(path = "heatmap_template.html")]
pub struct HeatmapTemplate {
    points_js: Vec<Vec<Vec<f64>>>,
    athlete_id: i64,
    sport_types: Vec<String>,
    selected_sports: Vec<String>,
    date_from: String,
    date_to: String,
    max_count: i32,
}

impl HeatmapTemplate {
    async fn new(state: &Arc<AppState>, params: HeatmapQueryParams) -> Self {
        let athlete_id = params.athlete_id;
        let selected_sports = params.sport_types;
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

        let points_js =
            build_standard_heat_map(&state, athlete_id, sport_filter, from_filter, to_filter).await;

        debug!("Routes found and heatmap build: {:?}", points_js.len());
        let max_count: i32 = points_js.len() as i32;
        Self {
            points_js,
            athlete_id,
            sport_types,
            selected_sports,
            date_from,
            date_to,
            max_count,
        }
    }
}

pub async fn get_template(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    let params = HeatmapQueryParams::from_raw(query.as_deref().unwrap_or(""));
    match params {
        Some(p) => HtmlTemplate(HeatmapTemplate::new(&state, p).await).into_response(),
        None => axum::http::StatusCode::BAD_REQUEST.into_response(),
    }
}

async fn build_standard_heat_map(
    state: &AppState,
    athlete_id: i64,
    sport_filter: Option<&[&str]>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Vec<Vec<Vec<f64>>> {
    let routes = ActivityStreamRepository::get_latlng_points_filtered(
        &state.db_pools,
        athlete_id,
        sport_filter,
        date_from,
        date_to,
    )
    .await
    .unwrap_or_else(|e| {
        log::error!("Failed to fetch latlng streams: {e}");
        vec![]
    });

    // Grid resolution ~100m at equator
    let precision = 1000.0;

    // Count how many routes pass through each grid cell
    let mut cell_counts: HashMap<(i64, i64), u32> = HashMap::new();
    for route in &routes {
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

    // Build output with [lat, lng, value] where value is 0-100
    routes
        .into_iter()
        .map(|route| {
            route
                .into_iter()
                .filter_map(|point| {
                    if point.len() >= 2 {
                        let cell = ((point[0] * precision) as i64, (point[1] * precision) as i64);
                        let count = *cell_counts.get(&cell).unwrap_or(&1) as f64;
                        Some(vec![point[0], point[1], count])
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

async fn build_heartRate_heat_map(
    state: &AppState,
    athlete_id: i64,
    sport_filter: Option<&[&str]>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Vec<Vec<Vec<f64>>> {
    let routes = ActivityStreamRepository::get_latlng_heart_rate_points_filtered(
        &state.db_pools,
        athlete_id,
        sport_filter,
        date_from,
        date_to,
    )
    .await
    .unwrap_or_else(|e| {
        log::error!("Failed to fetch latlng streams: {e}");
        HeartRateLatLngResults {
            heartrate: vec![],
            latlng: vec![],
        }
    });

    vec![]
}
