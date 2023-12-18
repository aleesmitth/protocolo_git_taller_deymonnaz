use rocket::tokio::task::spawn_blocking;
use rocket::{get, post, put};
use crate::commands::git_commands;
use crate::commands::git_commands::Command;
use crate::server::models::*;
use rocket::serde::json::Json;

use rocket::State;


use super::models::create;

#[get("/repos/<repo>/pulls")]
pub async fn get_repo_pull_request(state: &State<AppState>, repo: String) -> String {
    let mut options = PullRequestOptions::default();
    options.repo = Some(repo);
    match read(&options, &state.db_pool).await {
        Ok(pull_requests) => {
            // 4. Return an appropriate response
            format!("Pull request: {:?}", pull_requests)
        }
        Err(err) => {
            // Handle the error appropriately (log it, return an error response, etc.)
            format!("Error fetching pull request: {:?}", err)
        }
    }
}

#[get("/repos/<repo>/pulls/<pull_name>")]
pub async fn get_pull_request(state: &State<AppState>, repo: String, pull_name: String) -> String {
    let mut options = PullRequestOptions::default();
    options.repo = Some(repo);
    options.name = Some(pull_name);
    match read(&options, &state.db_pool).await {
        Ok(pull_requests) => {
            // 4. Return an appropriate response
            format!("Pull request: {:?}", pull_requests)
        }
        Err(err) => {
            // Handle the error appropriately (log it, return an error response, etc.)
            format!("Error fetching pull request: {:?}", err)
        }
    }
}

#[get("/repos/<repo>/pulls/<pull_name>/commits")]
pub async fn get_pull_request_commits(_state: &State<AppState>, repo: String, pull_name: String) -> String {
    let mut options = PullRequestOptions::default();
    options.repo = Some(repo);
    options.name = Some(pull_name);
    match read(&options, &_state.db_pool).await {
        Ok(pull_requests) => {
            // only one pull request should've been returned
            // assuming that only the first pull request is the valid one
            let head = pull_requests[0].head.clone();
            let base = pull_requests[0].base.clone();
            let commit_after_merge = pull_requests[0].commit_after_merge.clone();
            if let Some(commit) = commit_after_merge {
                match git_commands::Log::new().execute(Some(vec![&commit])) {
                    Ok(log) => format!("{}", log),
                    Err(err) => format!("Error fetching logs: {:?}", err)
                }
            } else {
                // TODO return los logs de head y base por separado
                format!("TODO")
            }
        }
        Err(err) => {
            // Handle the error appropriately (log it, return an error response, etc.)
            format!("Error fetching pull request: {:?}", err)
        }
    }
}

#[put("/repos/<repo>/pulls/<pull_name>/merge", format = "application/json")]
pub async fn put_merge(_state: &State<AppState>, repo: String, pull_name: String) -> String {
    let mut options = PullRequestOptions::default();
    options.repo = Some(repo);
    options.name = Some(pull_name);
    "TODO implement end point".to_string()
}

#[get("/init/<repo_name>")]
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
pub async fn post_pull_request(state: &State<AppState>, repo: String, pr: Json<PullRequest>) -> String {
    // 1. Extract data from the Json<PullRequestData> parameter
    let pull_request_data = pr.into_inner();

    // 2. Validate the extracted data
    if pull_request_data.name.is_empty() {
        return format!("Error: Pull request name cannot be empty.{}" ,repo);
    }
    match create(&pull_request_data, &state.db_pool).await {
        Ok(pull_request_id) => {
            // 4. Return an appropriate response
            format!("Pull request created successfully with Id: {}", pull_request_id)
        }
        Err(err) => {
            // Handle the error appropriately (log it, return an error response, etc.)
            format!("Error creating pull request: {:?}", err)
        }
    }
}


