use chrono::{ NaiveDate, NaiveDateTime, Utc };
use sqlx::PgPool;
use std::error::Error;
use sqlx::Row;
use sqlx::FromRow;
use serde::{Serialize, Deserialize};

// TODO make a constructor for this, so it validates data or whatever, is it cleaner?
// TODO we should use traits to support more than 1 model.
// TODO refactor to use macros so that queries are verified at compile time

pub struct AppState {
    pub db_pool: PgPool,
}

#[derive(Debug,FromRow,Serialize,Deserialize)]
pub struct PullRequest {
    pub _id: Option<i32>,
    pub name: String,
    pub repo: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>
}

impl PullRequest {
    pub fn new(id: Option<i32>, name: String, repo: String) -> Self {
        PullRequest {
            _id: id,
            name,
            repo,
            created_at: None
        }
    }
}
// TODO check for errors, refactor this and use table name in a .env var or constant
// TODO refactor to use transactions instead of the pool directly
pub async fn create(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<i32, Box<dyn Error>> {
    let query = "INSERT INTO pull_requests (name, repo) VALUES ($1, $2) RETURNING _id";
    let row = sqlx::query(query)
        .bind(&pull_request.name)
        .bind(&pull_request.repo)
        .fetch_one(pool)
        .await?;

    Ok(row.get("_id"))
}

pub async fn update(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE pull_requests SET name = $1, repo = $2 WHERE _id = $3";
    sqlx::query(query)
        .bind(&pull_request.name)
        .bind(&pull_request.repo)
        .bind(&pull_request._id.unwrap_or(1))
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn read(pool: &sqlx::PgPool) -> Result<Vec<PullRequest>, Box<dyn Error>> {
    let encoded_query = "SELECT _id, name, repo, created_at FROM pull_requests";
    let query = sqlx::query_as::<_, PullRequest>(encoded_query);

    let prs = query.fetch_all(pool).await?;

    Ok(prs)
}