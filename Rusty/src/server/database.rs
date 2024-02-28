extern crate rocket;
//use rusty::server::server_protocol::ServerProtocol;
use std::env;
use std::error::Error;
//use rocket::tokio::task::spawn_blocking;
//use std::thread;
use dotenv::dotenv;
use std::process::Command;
use sqlx::Row;

use crate::server::models::{self, PullRequest, PullRequestOptions};

pub struct Database;

impl Database{
    pub fn new() -> Self {
        Database {}
    }
    
    pub async fn run(&self) -> Result<sqlx::PgPool, Box<dyn std::error::Error>> {
        dotenv().ok();
    // Command to run the shell script
        let script_path = "create_database.sh";

        // Execute the shell script to create the database if it doesn't exist
        let status = Command::new("sh")
            .arg(script_path)
            .status()
            .expect("Failed to execute the script");

        if status.success() {
            println!("Script executed successfully");
        } else {
            println!("Script execution failed");
        }

        let url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        println!("url: {}", url);
        let pool = sqlx::postgres::PgPool::connect(&url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        println!("migrated");
        //let pr = PullRequest::new(None, "hello_world_branch".to_string(), "this_repo".to_string());

        //let pr_id = models::create(&pr, &pool).await?;
        //println!("created");
        //let pr = PullRequest::new(Some(pr_id), "updated_branch".to_string(), "this_repo".to_string());
        //models::update(&pr, &pool).await?;
        //println!("updated");
        let options = PullRequestOptions::default();
        let pr = read(&options, &pool).await?;
        println!("--small test--");
        println!("pr fetched from database: {:?}", pr);
        println!("--end small test--");

        Ok(pool)
    }
}

// TODO check for errors, refactor this and use table name in a .env var or constant
// TODO refactor to use transactions instead of the pool directly
pub async fn create(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<i32, Box<dyn Error>> {
    let query = "INSERT INTO pull_requests (repo, head, base) VALUES ($1, $2, $3) RETURNING _id";
    let row = sqlx::query(query)
        .bind(&pull_request.repo)
        .bind(&pull_request.head)
        .bind(&pull_request.base)
        .fetch_one(pool)
        .await?;

    Ok(row.get("_id"))
}

pub async fn update(pull_request: &PullRequest, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE pull_requests SET repo = $1, head = $2, base = $3 WHERE _id = $4";
    sqlx::query(query)
        .bind(&pull_request.repo)
        .bind(&pull_request.head)
        .bind(&pull_request.base)
        .bind(pull_request._id.unwrap_or(1))
        .execute(pool)
        .await?;

    Ok(())
}

// TODO fix this read query, it's very bad
pub async fn read(options: &PullRequestOptions, pool: &sqlx::PgPool) -> Result<Vec<PullRequest>, Box<dyn Error>> {
    let mut query = sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests");

    if let Some(repo) = &options.repo {
        if let Some(base) = &options.base {
            if let Some(head) = &options.head {
                query = sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests WHERE base = $1 AND head = $2 AND repo = $3")
                    .bind(base)
                    .bind(head)
                    .bind(repo);
            }
        } else {
            query = sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests WHERE repo = $1")
                .bind(repo);
        }
    } else if let Some(_id) = &options._id {
        query = sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests WHERE _id = $1")
            .bind(_id);
    }

    let prs = query.fetch_all(pool).await?;


    Ok(prs)
}
