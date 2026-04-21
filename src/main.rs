use axum::Router;
use chrono::Local;
use env_logger::Builder;
use log::{LevelFilter, debug, error, info};
use stravai_oxide::libs::repository::db_pool as DbPoolModule;
use stravai_oxide::{AppState, controllers};

use std::fs;
use std::io::Write;
use std::path::Path;
use std::{env, str::FromStr};
use stravai_oxide::libs::scheduler;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, logging_level())
        .init();

    let client_secret = match env::var("STRAVA_CLIENT_SECRET").ok().map(|id| {
        debug!("Using STRAVA_CLIENT_ID: {id}");
        id.trim().trim_matches('"').to_string()
    }) {
        Some(client_secret) => client_secret,
        None => {
            error!("Missing STRAVA_CLIENT_SECRET environment variable");
            return;
        }
    };

    let client_id: i32 = match env::var("STRAVA_CLIENT_ID").ok().and_then(|id| {
        debug!("Using STRAVA_CLIENT_ID: {id}");
        id.trim().trim_matches('"').parse().ok()
    }) {
        Some(id) => id,
        None => {
            error!("Missing STRAVA_CLIENT_ID environment variable");
            return;
        }
    };

    // Determine the current working directory for serving assets.
    let assets_path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            error!("Unable to find assets: {e}");
            return;
        }
    };

    // Create /config directory if it doesn't exist
    let config_dir = "/config";
    if !Path::new(config_dir).exists() {
        if let Err(e) = fs::create_dir_all(config_dir) {
            error!("Failed to create /config directory: {e}");
            return;
        }
    }

    // Initialize database
    let db_pools =
        match DbPoolModule::DbPool::connect_to_db("sqlite:/config/stravai.db?mode=rwc").await {
            Ok(pool) => pool,
            Err(error) => {
                error!("Failed to connect to database: {error}");
                return;
            }
        };

    // Create schema if it doesn't exist
    if let Err(e) = DbPoolModule::DbPool::init_db(&db_pools).await {
        error!("Failed to initialize database schema: {e}");
        return;
    }
    let app_state = AppState::new(client_id, client_secret, db_pools);

    // Start the background scheduler
    scheduler::start_scheduler(app_state.clone()).await;
    debug!("Scheduler started");

    // build our application with a route
    let app = Router::new()
        .merge(controllers::routes(app_state))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3400").await.unwrap();
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    info!("Application started")
}

fn logging_level() -> LevelFilter {
    let level: String = match env::var("LOGGING") {
        Ok(level) => level,
        Err(_) => return LevelFilter::Info,
    };

    LevelFilter::from_str(&level).unwrap_or(LevelFilter::Info)
}
