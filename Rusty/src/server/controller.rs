use rocket::tokio::task::spawn_blocking;
use rocket::{get, post, put, routes};
use crate::commands::git_commands;
use crate::commands::git_commands::{Command, Pull};
use crate::server::models::*;
use rocket::serde::json::Json;
use serde::Deserialize;
use serde::Serialize;
use rocket::State;
use sqlx::PgPool;
use crate::server::models;


use super::models::create;

#[get("/get/<repo_name>")]
pub async fn world(repo_name: &str) -> String {
    /*let repo_name_clone = repo_name.to_string(); // Clone the string
    let _vec = spawn_blocking(move || {
        if let Err(e) = git_commands::Init::new().execute(Some(vec![&repo_name_clone])) {
            println!("e {}",e);
        }
    }).await;*/
    format!("repo name: {}", repo_name)
}

#[put("/repos/init/<repo_name>")]
pub async fn init_repo(repo_name: &str) -> String {
    let repo_name_clone = repo_name.to_string(); // Clone the string
    let _vec = spawn_blocking(move || {
        if let Err(e) = git_commands::Init::new().execute(Some(vec![&repo_name_clone])) {
            println!("e {}",e);
        }
    }).await;
    format!("result: {:?}", _vec)
}

#[post("/repos/<repo>/pulls", format = "application/json", data = "<pr>")]
pub async fn new_pr(state: &State<AppState>, repo: String, pr: Json<PullRequest>) -> String {
    // 1. Extract data from the Json<PullRequestData> parameter
    let pull_request_data = pr.into_inner();

    // 2. Validate the extracted data
    if pull_request_data.name.is_empty() {
        return format!("Error: Pull request name cannot be empty.{}" ,repo);
    }
    match create(&pull_request_data, &state.db_pool).await {
        Ok(pull_request_id) => {
            // 4. Return an appropriate response
            format!("Pull request created successfully with ID: {}", pull_request_id)
        }
        Err(err) => {
            // Handle the error appropriately (log it, return an error response, etc.)
            format!("Error creating pull request: {:?}", err)
        }
    }
}

#[get("/y")]
pub async fn your_handler_function(state: &State<AppState>) -> String {
    // Access your database connection
    let db = &state.db_pool;
    let pr = PullRequest::new(None, "pushed from api".to_string());
    let pr_id = models::create(&pr, &db).await;

    // Perform database operations as needed
    // ...

    "Hello, World!".to_string()
}

