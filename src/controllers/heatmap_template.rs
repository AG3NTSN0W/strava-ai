use super::HtmlTemplate;
use crate::AppState;
use crate::libs::polyline::Polyline;
use crate::libs::repository::ActivityRepository;
use askama::Template;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use log::debug;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct HeatmapTemplateQueryParams {
    athlete_id: i64,
}

#[derive(Template)]
#[template(path = "heatmap_template.html")]
pub struct HeatmapTemplate {
    points_js: Vec<Vec<f64>>,
}

impl HeatmapTemplate {
    pub async fn new(
        State(state): State<Arc<AppState>>,
        query_params: HeatmapTemplateQueryParams,
    ) -> Self {
        let polylines =
            ActivityRepository::get_polyline_last_month_by_id(&state.db_pools, query_params.athlete_id)
                .await
                .unwrap_or_else(|e| {
                    log::error!("Failed to fetch athletes from database: {e}");
                    vec![]
                });

        debug!("polylines found: {:?}", polylines.len());

        Self {
            points_js: Polyline::new(polylines).points_js,
        }
    }
}

pub async fn get_template(
    state: State<Arc<AppState>>,
    query_params: Query<HeatmapTemplateQueryParams>,
) -> impl IntoResponse {
    HtmlTemplate(HeatmapTemplate::new(state, query_params.0).await)
}
