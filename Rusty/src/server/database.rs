extern crate rocket;
//use rusty::server::server_protocol::ServerProtocol;
use std::env;
//use rocket::tokio::task::spawn_blocking;
//use std::thread;
use dotenv::dotenv;
use std::process::Command;

use crate::server::models::{PullRequest, self, PullRequestOptions};

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
        let pr = models::read(&options, &pool).await?;
        println!("pr fetched from database: {:?}", pr);

        Ok(pool)
    }
}