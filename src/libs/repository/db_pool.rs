use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Error, Pool, Sqlite, SqlitePool};
use std::str::FromStr;

pub struct DbPool {}

impl DbPool {
    pub async fn connect_to_db(database_url: &str) -> Result<Pool<Sqlite>, Error> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .busy_timeout(std::time::Duration::from_secs(30));
        let pool = SqlitePool::connect_with(options).await?;
        Ok(pool)
    }

    pub async fn init_db(pool: &Pool<Sqlite>) -> Result<(), Error> {
        Self::init_athlete_db(pool).await?;
        Self::init_activity_db(pool).await?;
        Ok(())
    }

    pub async fn init_athlete_db(pool: &Pool<Sqlite>) -> Result<(), Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS athlete (
                id INTEGER PRIMARY KEY,
                firstname TEXT NOT NULL,
                lastname TEXT NOT NULL,
                premium BOOLEAN NOT NULL DEFAULT FALSE,
                refresh_token TEXT NOT NULL,
                prompt TEXT NOT NULL,
                auto_update BOOLEAN NOT NULL DEFAULT FALSE
            )"
        )
        .execute(pool)
        .await?;
        log::debug!("Athlete table initialized successfully");
        Ok(())
    }

    pub async fn init_activity_db(pool: &Pool<Sqlite>) -> Result<(), Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS activity (
                id INTEGER PRIMARY KEY,
                athlete_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                distance REAL NOT NULL,
                moving_time TEXT NOT NULL,
                elapsed_time TEXT NOT NULL,
                total_elevation_gain REAL,
                activity_type TEXT NOT NULL,
                sport_type TEXT NOT NULL,
                start_date_local TEXT NOT NULL,
                achievement_count INTEGER,
                average_speed REAL,
                max_speed REAL,
                average_watts REAL,
                kilojoules REAL,
                average_heartrate REAL,
                max_heartrate REAL,
                elev_high REAL,
                elev_low REAL,
                pr_count INTEGER,
                FOREIGN KEY(athlete_id) REFERENCES athlete(id) ON DELETE CASCADE
            )"
        )
        .execute(pool)
        .await?;
        log::debug!("Activity table initialized successfully");
        Ok(())
    }
}
