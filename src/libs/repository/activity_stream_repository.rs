use crate::libs::StravAIError;
use crate::libs::models::activity_stream::ActivityStream;
use serde::Serialize;
use sqlx::sqlite::SqliteRow;
use sqlx::{Error, Pool, Row, Sqlite};
use std::collections::HashMap;

pub struct HeartRateLatLngResults {
    pub latlng: Vec<Vec<Vec<f64>>>,
    pub heartrate: Vec<Vec<f64>>,
}

pub struct AltitudeLatLngResults {
    pub latlng: Vec<Vec<Vec<f64>>>,
    pub altitude: Vec<Vec<f64>>,
}

pub struct VelocityLatLngResults {
    pub latlng: Vec<Vec<Vec<f64>>>,
    pub velocity: Vec<Vec<f64>>,
}

pub struct ActivityStreamResults<T> {
    pub data: T,
    pub activity_data: Vec<ActivityDataResults>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityDataResults {
    pub distance: f32,
    pub elapsed_time: String,
    pub total_elevation_gain: f32,
    pub average_heartrate: i32,
}

pub struct ActivityStreamRepository;

impl ActivityStreamRepository {
    pub async fn create(pool: &Pool<Sqlite>, stream: &ActivityStream) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO activity_stream (activity_id, stream_type, data, series_type, original_size, resolution)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT (activity_id, stream_type) DO UPDATE SET
                data = excluded.data,
                series_type = excluded.series_type,
                original_size = excluded.original_size,
                resolution = excluded.resolution"
        )
        .bind(stream.activity_id)
        .bind(&stream.stream_type)
        .bind(&stream.data)
        .bind(&stream.series_type)
        .bind(stream.original_size)
        .bind(&stream.resolution)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_by_activity_id(
        pool: &Pool<Sqlite>,
        activity_id: i64,
    ) -> Result<Vec<ActivityStream>, Error> {
        sqlx::query_as::<_, ActivityStream>(
            "SELECT id, activity_id, stream_type, data, series_type, original_size, resolution
             FROM activity_stream WHERE activity_id = ?",
        )
        .bind(activity_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_by_activity_id_and_type(
        pool: &Pool<Sqlite>,
        activity_id: i64,
        stream_type: &str,
    ) -> Result<Option<ActivityStream>, Error> {
        sqlx::query_as::<_, ActivityStream>(
            "SELECT id, activity_id, stream_type, data, series_type, original_size, resolution
             FROM activity_stream WHERE activity_id = ? AND stream_type = ?",
        )
        .bind(activity_id)
        .bind(stream_type)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete_by_activity_id(pool: &Pool<Sqlite>, activity_id: i64) -> Result<(), Error> {
        sqlx::query("DELETE FROM activity_stream WHERE activity_id = ?")
            .bind(activity_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_latlng_points_filtered(
        pool: &Pool<Sqlite>,
        athlete_id: i64,
        sport_types: Option<&[&str]>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ActivityStreamResults<Vec<Vec<Vec<f64>>>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut sql = String::from(
            "SELECT s.data,
                    ROUND(a.distance, 2) as distance,
                    a.elapsed_time,
                    ROUND(a.total_elevation_gain, 2) as total_elevation_gain,
                    CAST(a.average_heartrate AS INTEGER) as average_heartrate
                FROM activity_stream s
                JOIN activity a ON a.id = s.activity_id
                WHERE a.athlete_id = ? AND s.stream_type = 'latlng'",
        );

        if let Some(types) = sport_types {
            if !types.is_empty() {
                let placeholders: Vec<&str> = types.iter().map(|_| "?").collect();
                sql.push_str(&format!(
                    " AND a.sport_type IN ({})",
                    placeholders.join(",")
                ));
            }
        }
        if date_from.is_some() {
            sql.push_str(" AND a.start_date_local >= ?");
        }
        if date_to.is_some() {
            sql.push_str(" AND a.start_date_local <= ?");
        }

        let mut query = sqlx::query(&sql).bind(athlete_id);

        if let Some(types) = sport_types {
            for t in types {
                query = query.bind(*t);
            }
        }
        if let Some(from) = date_from {
            query = query.bind(from);
        }
        if let Some(to) = date_to {
            query = query.bind(to);
        }

        let rows = query.fetch_all(pool).await?;
        let mut result: Vec<Vec<Vec<f64>>> = Vec::new();
        let mut activity_data: Vec<ActivityDataResults> = Vec::new();

        for row in rows {
            let data: String = row.get("data");
            let parsed: Vec<Vec<f64>> = serde_json::from_str(&data)?;
            result.push(parsed);
            activity_data.push(Self::get_activity_data(&row))
        }

        Ok(ActivityStreamResults {
            data: result,
            activity_data,
        })
    }

    pub async fn get_latlng_heart_rate_points_filtered(
        pool: &Pool<Sqlite>,
        athlete_id: i64,
        sport_types: Option<&[&str]>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ActivityStreamResults<HeartRateLatLngResults>, Box<dyn std::error::Error + Send + Sync>> {
        let mut sql = String::from(
            "SELECT s.activity_id, s.data, s.stream_type,
                    ROUND(a.distance, 2) as distance,
                    a.elapsed_time,
                    ROUND(a.total_elevation_gain, 2) as total_elevation_gain,
                    CAST(a.average_heartrate AS INTEGER) as average_heartrate
                FROM activity_stream s
             JOIN activity a ON a.id = s.activity_id
             WHERE a.athlete_id = ? AND s.stream_type IN ('latlng', 'heartrate')",
        );

        if let Some(types) = sport_types {
            if !types.is_empty() {
                let placeholders: Vec<&str> = types.iter().map(|_| "?").collect();
                sql.push_str(&format!(
                    " AND a.sport_type IN ({})",
                    placeholders.join(",")
                ));
            }
        }
        if date_from.is_some() {
            sql.push_str(" AND a.start_date_local >= ?");
        }
        if date_to.is_some() {
            sql.push_str(" AND a.start_date_local <= ?");
        }
        sql.push_str(" ORDER BY s.activity_id");

        let mut query = sqlx::query(&sql).bind(athlete_id);

        if let Some(types) = sport_types {
            for t in types {
                query = query.bind(*t);
            }
        }
        if let Some(from) = date_from {
            query = query.bind(from);
        }
        if let Some(to) = date_to {
            query = query.bind(to);
        }

        let rows = query.fetch_all(pool).await?;

        // Group streams by activity_id to keep latlng and heartrate paired
        let mut activity_latlng: HashMap<i64, Vec<Vec<f64>>> = HashMap::new();
        let mut activity_hr: HashMap<i64, Vec<f64>> = HashMap::new();

        let mut activity_data = Vec::new();

        for row in rows {
            let activity_id: i64 = row.get("activity_id");
            let stream_type: &str = row.get("stream_type");
            let data: String = row.get("data");
            if stream_type == "heartrate" {
                let parsed: Vec<f64> = serde_json::from_str(&data)?;
                activity_hr.insert(activity_id, parsed);
            } else if stream_type == "latlng" {
                let parsed: Vec<Vec<f64>> = serde_json::from_str(&data)?;
                activity_latlng.insert(activity_id, parsed);
                activity_data.push(Self::get_activity_data(&row));
            }

        }

        // Only include activities that have both streams
        let mut latlng: Vec<Vec<Vec<f64>>> = Vec::new();
        let mut heartrate: Vec<Vec<f64>> = Vec::new();

        for (id, ll) in activity_latlng {
            if let Some(hr) = activity_hr.remove(&id) {
                latlng.push(ll);
                heartrate.push(hr);
            }
        }

        Ok(ActivityStreamResults {
            data: HeartRateLatLngResults { heartrate, latlng },
            activity_data
        })
    }

    pub async fn get_latlng_altitude_points_filtered(
        pool: &Pool<Sqlite>,
        athlete_id: i64,
        sport_types: Option<&[&str]>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ActivityStreamResults<AltitudeLatLngResults>, Box<dyn std::error::Error + Send + Sync>> {
        let mut sql = String::from(
            "SELECT s.activity_id, s.data, s.stream_type,
                    ROUND(a.distance, 2) as distance,
                    a.elapsed_time,
                    ROUND(a.total_elevation_gain, 2) as total_elevation_gain,
                    CAST(a.average_heartrate AS INTEGER) as average_heartrate
                FROM activity_stream s
             JOIN activity a ON a.id = s.activity_id
             WHERE a.athlete_id = ? AND s.stream_type IN ('latlng', 'altitude')",
        );

        if let Some(types) = sport_types {
            if !types.is_empty() {
                let placeholders: Vec<&str> = types.iter().map(|_| "?").collect();
                sql.push_str(&format!(
                    " AND a.sport_type IN ({})",
                    placeholders.join(",")
                ));
            }
        }
        if date_from.is_some() {
            sql.push_str(" AND a.start_date_local >= ?");
        }
        if date_to.is_some() {
            sql.push_str(" AND a.start_date_local <= ?");
        }
        sql.push_str(" ORDER BY s.activity_id");

        let mut query = sqlx::query(&sql).bind(athlete_id);

        if let Some(types) = sport_types {
            for t in types {
                query = query.bind(*t);
            }
        }
        if let Some(from) = date_from {
            query = query.bind(from);
        }
        if let Some(to) = date_to {
            query = query.bind(to);
        }

        let rows = query.fetch_all(pool).await?;

        let mut activity_latlng: HashMap<i64, Vec<Vec<f64>>> = HashMap::new();
        let mut activity_alt: HashMap<i64, Vec<f64>> = HashMap::new();
        let mut activity_data = Vec::new();

        for row in rows {
            let activity_id: i64 = row.get("activity_id");
            let stream_type: &str = row.get("stream_type");
            let data: String = row.get("data");
            if stream_type == "altitude" {
                let parsed: Vec<f64> = serde_json::from_str(&data)?;
                activity_alt.insert(activity_id, parsed);
            } else if stream_type == "latlng" {
                let parsed: Vec<Vec<f64>> = serde_json::from_str(&data)?;
                activity_latlng.insert(activity_id, parsed);
                activity_data.push(Self::get_activity_data(&row));
            }

        }

        let mut latlng: Vec<Vec<Vec<f64>>> = Vec::new();
        let mut altitude: Vec<Vec<f64>> = Vec::new();

        for (id, ll) in activity_latlng {
            if let Some(alt) = activity_alt.remove(&id) {
                latlng.push(ll);
                altitude.push(alt);
            }
        }

        Ok(ActivityStreamResults {
            data: AltitudeLatLngResults { altitude, latlng },
            activity_data
        })
    }

    pub async fn get_latlng_velocity_points_filtered(
        pool: &Pool<Sqlite>,
        athlete_id: i64,
        sport_types: Option<&[&str]>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ActivityStreamResults<VelocityLatLngResults>, Box<dyn std::error::Error + Send + Sync>> {
        let mut sql = String::from(
            "SELECT s.activity_id, s.data, s.stream_type,
                    ROUND(a.distance, 2) as distance,
                    a.elapsed_time,
                    ROUND(a.total_elevation_gain, 2) as total_elevation_gain,
                    CAST(a.average_heartrate AS INTEGER) as average_heartrate
                FROM activity_stream s
             JOIN activity a ON a.id = s.activity_id
             WHERE a.athlete_id = ? AND s.stream_type IN ('latlng', 'velocity_smooth')",
        );

        if let Some(types) = sport_types {
            if !types.is_empty() {
                let placeholders: Vec<&str> = types.iter().map(|_| "?").collect();
                sql.push_str(&format!(
                    " AND a.sport_type IN ({})",
                    placeholders.join(",")
                ));
            }
        }
        if date_from.is_some() {
            sql.push_str(" AND a.start_date_local >= ?");
        }
        if date_to.is_some() {
            sql.push_str(" AND a.start_date_local <= ?");
        }
        sql.push_str(" ORDER BY s.activity_id");

        let mut query = sqlx::query(&sql).bind(athlete_id);

        if let Some(types) = sport_types {
            for t in types {
                query = query.bind(*t);
            }
        }
        if let Some(from) = date_from {
            query = query.bind(from);
        }
        if let Some(to) = date_to {
            query = query.bind(to);
        }

        let rows = query.fetch_all(pool).await?;

        let mut activity_latlng: HashMap<i64, Vec<Vec<f64>>> = HashMap::new();
        let mut activity_vel: HashMap<i64, Vec<f64>> = HashMap::new();
        let mut activity_data = Vec::new();

        for row in rows {
            let activity_id: i64 = row.get("activity_id");
            let stream_type: &str = row.get("stream_type");
            let data: String = row.get("data");
            if stream_type == "velocity_smooth" {
                let parsed: Vec<f64> = serde_json::from_str(&data)?;
                activity_vel.insert(activity_id, parsed);
            } else if stream_type == "latlng" {
                let parsed: Vec<Vec<f64>> = serde_json::from_str(&data)?;
                activity_latlng.insert(activity_id, parsed);
                activity_data.push(Self::get_activity_data(&row));
            }

        }

        let mut latlng: Vec<Vec<Vec<f64>>> = Vec::new();
        let mut velocity: Vec<Vec<f64>> = Vec::new();

        for (id, ll) in activity_latlng {
            if let Some(vel) = activity_vel.remove(&id) {
                latlng.push(ll);
                velocity.push(vel);
            }
        }

        Ok(ActivityStreamResults {
            data: VelocityLatLngResults { velocity, latlng },
            activity_data
        })
    }

    pub async fn get_latlng_points_by_id(
        pool: &Pool<Sqlite>,
        athlete_id: i64,
        activity_id: i64,
    ) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error + Send + Sync>> {
        let sql = String::from(
            "SELECT s.data FROM activity_stream s
            JOIN activity a ON a.id = s.activity_id
             WHERE a.athlete_id = ? AND s.activity_id = ?  AND s.stream_type = 'latlng'",
        );

        let row = sqlx::query(&sql)
            .bind(athlete_id)
            .bind(activity_id)
            .fetch_optional(pool)
            .await?;

        match row {
            None => Err(Box::new(StravAIError("Map data not found".into()))),
            Some(row) => {
                let data: String = row.get("data");
                Ok(serde_json::from_str(&data)?)
            }
        }
    }

    fn get_activity_data(row: &SqliteRow) -> ActivityDataResults {
        ActivityDataResults {
            distance: row.get("distance"),
            elapsed_time: row.get("elapsed_time"),
            total_elevation_gain: row.get("total_elevation_gain"),
            average_heartrate: row.get("average_heartrate"),
        }
    }
}
