use chrono::{ NaiveDate, NaiveDateTime, Utc };
use std::error::Error;
pub struct PullRequest {
    pub id: Option<i32>,
    pub name: String
}

pub async fn create(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "INSERT INTO pull_requests (name) VALUES ($1)";
    sqlx::query(query)
        .bind(&pull_request.name)
        .execute(pool)
        .await?;

    Ok(())
}