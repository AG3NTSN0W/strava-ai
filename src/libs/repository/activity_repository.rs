use crate::libs::models::athlete_activity::AthleteActivity;
use sqlx::{Error, Pool, Row, Sqlite};

pub struct ActivityRepository;

impl ActivityRepository {
    /// Create a new activity in the database
    pub async fn create(pool: &Pool<Sqlite>, activity: &AthleteActivity) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO activity (
                id, athlete_id, name, description, distance, moving_time, elapsed_time, total_elevation_gain,
                activity_type, sport_type, start_date_local, achievement_count,
                average_speed, max_speed, average_watts, kilojoules, average_heartrate,
                max_heartrate, elev_high, elev_low, pr_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT (id) DO NOTHING;"
        )
        .bind(activity.id)
        .bind(activity.athlete_id)
        .bind(&activity.name)
        .bind(&activity.description)
        .bind(activity.distance)
        .bind(&activity.moving_time)
        .bind(&activity.elapsed_time)
        .bind(activity.total_elevation_gain)
        .bind(&activity.activity_type)
        .bind(&activity.sport_type)
        .bind(&activity.start_date_local)
        .bind(activity.achievement_count)
        .bind(activity.average_speed)
        .bind(activity.max_speed)
        .bind(activity.average_watts)
        .bind(activity.kilojoules)
        .bind(activity.average_heartrate)
        .bind(activity.max_heartrate)
        .bind(activity.elev_high)
        .bind(activity.elev_low)
        .bind(activity.pr_count)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Get an activity by ID
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<AthleteActivity>, Error> {
        sqlx::query_as::<_, AthleteActivity>(
            "SELECT id, athlete_id, name, description, distance, moving_time, elapsed_time, total_elevation_gain,
                    activity_type, sport_type, start_date_local, achievement_count,
                    average_speed, max_speed, average_watts, kilojoules, average_heartrate,
                    max_heartrate, elev_high, elev_low, pr_count FROM activity WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Get all activities
    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<AthleteActivity>, Error> {
        sqlx::query_as::<_, AthleteActivity>(
            "SELECT id, athlete_id, name, description, distance, moving_time, elapsed_time, total_elevation_gain,
                    activity_type, sport_type, start_date_local, achievement_count,
                    average_speed, max_speed, average_watts, kilojoules, average_heartrate,
                    max_heartrate, elev_high, elev_low, pr_count FROM activity"
        )
        .fetch_all(pool)
        .await
    }

    /// Get all activities for a specific athlete
    pub async fn get_by_athlete_id(
        pool: &Pool<Sqlite>,
        athlete_id: i64,
    ) -> Result<Vec<AthleteActivity>, Error> {
        sqlx::query_as::<_, AthleteActivity>(
            "SELECT id, athlete_id, name, description, distance, moving_time, elapsed_time, total_elevation_gain,
                    activity_type, sport_type, start_date_local, achievement_count,
                    average_speed, max_speed, average_watts, kilojoules, average_heartrate,
                    max_heartrate, elev_high, elev_low, pr_count FROM activity WHERE athlete_id = ? ORDER BY start_date_local DESC"
        )
        .bind(athlete_id)
        .fetch_all(pool)
        .await
    }

    /// Update an existing activity
    pub async fn update(pool: &Pool<Sqlite>, activity: &AthleteActivity) -> Result<(), Error> {
        sqlx::query(
            "UPDATE activity SET
                name = ?, description = ?, distance = ?, moving_time = ?, elapsed_time = ?,
                total_elevation_gain = ?, activity_type = ?, sport_type = ?,
                start_date_local = ?, achievement_count = ?, average_speed = ?,
                max_speed = ?, average_watts = ?, kilojoules = ?, average_heartrate = ?,
                max_heartrate = ?, elev_high = ?, elev_low = ?, pr_count = ?
             WHERE id = ?",
        )
        .bind(&activity.name)
        .bind(&activity.description)
        .bind(activity.distance)
        .bind(&activity.moving_time)
        .bind(&activity.elapsed_time)
        .bind(activity.total_elevation_gain)
        .bind(&activity.activity_type)
        .bind(&activity.sport_type)
        .bind(&activity.start_date_local)
        .bind(activity.achievement_count)
        .bind(activity.average_speed)
        .bind(activity.max_speed)
        .bind(activity.average_watts)
        .bind(activity.kilojoules)
        .bind(activity.average_heartrate)
        .bind(activity.max_heartrate)
        .bind(activity.elev_high)
        .bind(activity.elev_low)
        .bind(activity.pr_count)
        .bind(activity.id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Update an existing activity description
    pub async fn update_description(
        pool: &Pool<Sqlite>,
        description: &str,
        activity_id: i64,
    ) -> Result<(), Error> {
        sqlx::query("UPDATE activity SET description = ? WHERE id = ?")
            .bind(description)
            .bind(activity_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Update an existing activity description and title
    pub async fn update_description_and_name(
        pool: &Pool<Sqlite>,
        description: &str,
        name: &str,
        activity_id: i64,
    ) -> Result<(), Error> {
        sqlx::query("UPDATE activity SET description = ?, name = ?  WHERE id = ?")
            .bind(description)
            .bind(name)
            .bind(activity_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Delete an activity by ID
    pub async fn delete(pool: &Pool<Sqlite>, id: i64) -> Result<(), Error> {
        sqlx::query("DELETE FROM activity WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Delete all activities for a specific athlete
    pub async fn delete_by_athlete_id(pool: &Pool<Sqlite>, athlete_id: i64) -> Result<(), Error> {
        sqlx::query("DELETE FROM activity WHERE athlete_id = ?")
            .bind(athlete_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Check if an activity exists by ID
    pub async fn exists(pool: &Pool<Sqlite>, id: i64) -> Result<bool, Error> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM activity WHERE id = ?)")
            .bind(id)
            .fetch_one(pool)
            .await?;

        let i: i32 = row.get(0);
        Ok(i >= 1)
    }
}
