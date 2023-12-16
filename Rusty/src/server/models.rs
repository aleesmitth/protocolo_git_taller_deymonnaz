use chrono::{ NaiveDate, NaiveDateTime, Utc };
use std::error::Error;
use sqlx::Row;
use sqlx::FromRow;

// TODO make a constructor for this, so it validates data or whatever, is it cleaner?
// TODO we should use traits to support more than 1 model.
// TODO refactor to use macros so that queries are verified at compile time
#[derive(Debug,FromRow)]
pub struct PullRequest {
    pub id: Option<i32>,
    pub name: String
}
// TODO check for errors, refactor this and use table name in a .env var or constant
// TODO refactor to use transactions instead of the pool directly
pub async fn create(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "INSERT INTO pull_requests (name) VALUES ($1)";
    sqlx::query(query)
        .bind(&pull_request.name)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE pull_requests SET name = $1 WHERE id = $2";
    sqlx::query(query)
        .bind(&pull_request.name)
        .bind(&pull_request.id.unwrap_or(1))
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn read(pool: &sqlx::PgPool) -> Result<Vec<PullRequest>, Box<dyn Error>> {
    let encoded_query = "SELECT id, name, created_at FROM pull_requests";
    let query = sqlx::query_as::<_, PullRequest>(encoded_query);

    let prs = query.fetch_all(pool).await?;

    Ok(prs)
}