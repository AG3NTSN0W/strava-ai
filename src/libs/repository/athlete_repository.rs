use crate::libs::models::athlete::Athlete;
use sqlx::{Error, Pool, Row, Sqlite};

pub struct AthleteRepository;

impl AthleteRepository {
    /// Create a new athlete in the database, or update if the ID already exists
    pub async fn create(pool: &Pool<Sqlite>, athlete: &Athlete) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO athlete (id, firstname, lastname, premium, refresh_token, auto_update, prompt)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(athlete.id)
        .bind(&athlete.firstname)
        .bind(&athlete.lastname)
        .bind(athlete.premium)
        .bind(&athlete.refresh_token)
        .bind(athlete.auto_update)
        .bind(&athlete.prompt)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Get an athlete by ID
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<Athlete>, Error> {
        sqlx::query_as::<_, Athlete>(
            "SELECT id, firstname, lastname, premium, refresh_token, auto_update, prompt FROM athlete WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Get all athletes
    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<Athlete>, Error> {
        sqlx::query_as::<_, Athlete>(
            "SELECT id, firstname, lastname, premium, refresh_token, auto_update, prompt FROM athlete",
        )
        .fetch_all(pool)
        .await
    }

    /// Update an existing athlete
    pub async fn update(pool: &Pool<Sqlite>, athlete: &Athlete) -> Result<(), Error> {
        sqlx::query(
            "UPDATE athlete SET firstname = ?, lastname = ?, premium = ?, refresh_token = ?, auto_update = ?, prompt =?
             WHERE id = ?"
        )
        .bind(&athlete.firstname)
        .bind(&athlete.lastname)
        .bind(athlete.premium)
        .bind(&athlete.refresh_token)
        .bind(athlete.auto_update)
        .bind(&athlete.prompt)
        .bind(athlete.id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Update athlete refresh_token
    pub async fn update_refresh_token(
        pool: &Pool<Sqlite>,
        athlete_id: i64,
        refresh_token: &str,
    ) -> Result<(), Error> {
        sqlx::query("UPDATE athlete SET refresh_token = ? WHERE id = ?")
            .bind(athlete_id)
            .bind(refresh_token)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update_settings(
        pool: &Pool<Sqlite>,
        athlete_id: i64,
        prompt: &str,
        auto_update: bool
    ) -> Result<(), Error> {
        sqlx::query("UPDATE athlete SET prompt = ?, auto_update = ? WHERE id = ?")
            .bind(prompt)
            .bind(auto_update)
            .bind(athlete_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Delete an athlete by ID
    pub async fn delete(pool: &Pool<Sqlite>, id: i64) -> Result<(), Error> {
        sqlx::query("DELETE FROM athlete WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Check if an athlete exists by ID
    pub async fn exists(pool: &Pool<Sqlite>, id: i64) -> Result<bool, Error> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM athlete WHERE id = ?)")
            .bind(id)
            .fetch_one(pool)
            .await?;

        let i: i32 = row.get(0);
        Ok(i >= 1)
    }

    /// Get all athletes with auto_update enabled
    pub async fn get_all_with_auto_update(pool: &Pool<Sqlite>) -> Result<Vec<Athlete>, Error> {
        sqlx::query_as::<_, Athlete>(
            "SELECT id, firstname, lastname, premium, refresh_token, auto_update, prompt FROM athlete WHERE auto_update = 1"
        )
        .fetch_all(pool)
        .await
    }

    /// Update only the auto_update field for an athlete
    pub async fn update_auto_update(
        pool: &Pool<Sqlite>,
        id: i64,
        auto_update: bool,
    ) -> Result<(), Error> {
        sqlx::query("UPDATE athlete SET auto_update = ? WHERE id = ?")
            .bind(auto_update)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
