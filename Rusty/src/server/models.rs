use chrono;
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

#[derive(Debug, Default)]
pub struct PullRequestOptions {
    pub _id: Option<i32>,
    pub name: Option<String>,
    pub repo: Option<String>,
    pub head: Option<String>,  // Add the "head" field
    pub base: Option<String>,  // Add the "base" field
    pub created_at: Option<chrono::DateTime<chrono::Utc>>
}
#[derive(Debug,FromRow,Serialize,Deserialize)]
pub struct PullRequest {
    pub _id: Option<i32>,
    pub name: String,
    pub repo: String,
    pub head: String,  // Add the "head" field
    pub base: String,  // Add the "base" field
    pub created_at: Option<chrono::DateTime<chrono::Utc>>
}

impl PullRequest {
    pub fn new(name: String, repo: String, head: String, base: String) -> Self {
        PullRequest {
            _id: None,
            name,
            repo,
            head,
            base,
            created_at: None
        }
    }
}
// TODO check for errors, refactor this and use table name in a .env var or constant
// TODO refactor to use transactions instead of the pool directly
pub async fn create(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<i32, Box<dyn Error>> {
    let query = "INSERT INTO pull_requests (name, repo, head, base) VALUES ($1, $2, $3, $4) RETURNING _id";
    let row = sqlx::query(query)
        .bind(&pull_request.name)
        .bind(&pull_request.repo)
        .bind(&pull_request.head)
        .bind(&pull_request.base)
        .fetch_one(pool)
        .await?;

    Ok(row.get("_id"))
}

pub async fn update(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE pull_requests SET name = $1, repo = $2, head = $3, base = $4 WHERE _id = $3";
    sqlx::query(query)
        .bind(&pull_request.name)
        .bind(&pull_request.repo)
        .bind(&pull_request.head)
        .bind(&pull_request.base)
        .bind(pull_request._id.unwrap_or(1))
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn read(options: &PullRequestOptions, pool: &sqlx::PgPool) -> Result<Vec<PullRequest>, Box<dyn Error>> {
    let mut query = sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests");

    if let Some(repo) = &options.repo {
        if let Some(name) = &options.name {
            query = sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests WHERE name = $1 AND repo = $2")
                .bind(name)
                .bind(repo);
        } else {
            query = sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests WHERE repo = $1")
                .bind(repo);
        }
    } else if let Some(_id) = &options._id {
        query = sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests WHERE _id = $1").bind(_id);
    }

    let prs = query.fetch_all(pool).await?;


    Ok(prs)
}