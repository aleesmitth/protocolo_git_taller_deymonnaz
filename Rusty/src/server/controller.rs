use chrono::format;
use rocket::tokio::task::spawn_blocking;
use rocket::{get, post, put};
use crate::commands::git_commands;
use crate::commands::git_commands::Command;
use crate::server::models::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use rocket::State;


use super::models::create;

#[openapi(skip)]
#[get("/")]
pub async fn test_api(state: &State<AppState>) -> String {
    format!("Hello world")
}

/// # Get repo's pull requests
///
/// Returns all pull requests in the specified repo.
#[openapi(tag = "Pull Requests")]
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

/// # Get a specific pull request
///
/// Returns the specified pull request that's in the specified repo.
#[openapi(tag = "Pull Requests")]
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

/// # Get all pull request's commits
///
/// Returns all the commits from the specified pull request that's in the specified repo.
#[openapi(tag = "Pull Requests")]
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
                    Ok(log) => format!("Commit log: {}", log),
                    Err(err) => format!("Error fetching logs: {:?}", err)
                }
            } else {
                // TODO return los logs de head y base por separado
                let log_head: Vec<&str> = head.split(':').collect();
                if let Ok(hash_log_head) = git_commands::Log::new().execute(Some(vec![&log_head[1]])){
                    if let Ok(log_base) =  git_commands::Log::new().execute(Some(vec![&base])){
                        format!("Head log: {:?}\n Base log: {}", hash_log_head, log_base)
                    }
                    else {
                            format!("Error fetching logs")
                    }
                }
                else {
                    format!("Error fetching logs")
                }
                
            }
        }
        Err(err) => {
            // Handle the error appropriately (log it, return an error response, etc.)
            format!("Error fetching pull request: {:?}", err)
        }
    }
}

/// # Merge a pull request
///
/// Merges a pull request into the base branch.
#[openapi(tag = "Pull Requests")]
#[put("/repos/<repo>/pulls/<pull_name>/merge", format = "application/json")]
pub fn put_merge(_state: &State<AppState>, repo: String, pull_name: String) -> String {
    let mut options = PullRequestOptions::default();
    options.repo = Some(repo);
    options.name = Some(pull_name);
    "TODO implement end point".to_string()
}

/// # git init a repo for testing only
#[openapi(tag = "Pull Requests")]
#[get("/init/<repo>")]
pub async fn init_repo(repo: &str) -> String {
    let repo_name_clone = repo.to_string(); // Clone the string
    let _vec = spawn_blocking(move || {
        if let Err(e) = git_commands::Init::new().execute(Some(vec![&repo_name_clone])) {
            println!("e {}",e);
        }
    }).await;
    format!("result: {:?}", _vec)
}

/// # Create a pull request
#[openapi(tag = "Pull Requests")]
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
            /*let _vec = spawn_blocking(move || {
                if let Err(e) = git_commands::Merge::new().execute(Some(vec!["segunda_branch"])) {
                    println!("e {}",e);
                }
            });
            format!("result: {:?}", _vec);*/
            // 4. Return an appropriate response
            format!("Pull request created successfully with Id: {}", pull_request_id)
        }
        Err(err) => {
            // Handle the error appropriately (log it, return an error response, etc.)
            format!("Error creating pull request: {:?}", err)
        }
    }
}


