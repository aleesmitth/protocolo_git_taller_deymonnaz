use std::env;
use std::error::Error;
use rocket::tokio::task::spawn_blocking;
use rocket::{get, post, put};
use crate::commands::git_commands::{self, Merge};
use crate::commands::git_commands::{Command, PathHandler};
use crate::server::models::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use rocket::State;
use rocket::http::Status;
use rocket::response::status::NotFound;


use super::database::{create, read, update};

#[openapi(skip)]
#[get("/")]
pub async fn test_api(_state: &State<AppState>) -> String {
    "Hello world".to_string()
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
#[get("/repos/<repo>/pulls/<pull_number>")]
pub async fn get_pull_request(state: &State<AppState>, repo: String, pull_number: i32) -> String {
    let mut options = PullRequestOptions::default();
    options.repo = Some(repo);
    options._id = Some(pull_number);
    match read(&options, &state.db_pool).await {
        Ok(pull_requests) => {
            // 4. Return an appropriate response
            format!("{:?}", pull_requests)
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
#[get("/repos/<repo>/pulls/<pull_number>/commits")]
pub async fn get_pull_request_commits(state: &State<AppState>, repo: String, pull_number: i32) -> String {
    let mut options = PullRequestOptions::default();
    options.repo = Some(repo);
    options._id = Some(pull_number);
    match read(&options, &state.db_pool).await {
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
                        "Error fetching logs".to_string()
                    }
                }
                else {
                    "Error fetching logs".to_string()
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
// #[get("/repos/<repo>/pulls/<pull_number>/merge", format = "application/json", data = "<pr>")]
/// Merges a pull request into the base branch.
#[openapi(tag = "Pull Requests")]
#[put("/repos/<repo>/pulls/<pull_number>")]
pub async fn put_merge(state: &State<AppState>, repo: String, pull_number: i32) -> Result<String, NotFound<String>> {
    println!("inside put");
    let mut options = PullRequestOptions::default();
    options.repo = Some(repo.clone());
    options._id = Some(pull_number);

    let pull_requests = match read(&options, &state.db_pool).await {
        Ok(pull_requests) => pull_requests,
        Err(err) => {
            println!("error reading pr");
            return Err(NotFound(err.to_string()))
        }
    };
    let mut merged_pull_request = pull_requests[0].clone();
    println!("pull request: {:?}", pull_requests);
    if pull_requests.is_empty(){
        return Err(NotFound("Pull request came back empty".to_string()))
    }

    println!("merging");
    let merge = spawn_blocking(move || {
        let base_repo_path = PathHandler::get_relative_path("");

        PathHandler::set_relative_path(&format!("{}{}/", base_repo_path, repo));
        let merge = Merge::new().execute(Some(vec![&pull_requests[0].head, &pull_requests[0].base]))
            .map(|result| result.to_string())
            .map_err(|e| e.to_string());
        PathHandler::set_relative_path(&base_repo_path);
        merge
    }).await;
    println!("merged");
    match merge {
        // all good
        Ok(Ok(new_commit)) => {
            merged_pull_request.commit_after_merge = Some(new_commit.clone());
            match update(&merged_pull_request, &state.db_pool).await {
                Ok(_) => Ok(new_commit),
                Err(err) => {
                    return Err(NotFound("Error updating the PR in the database after merge".to_string()))
                }
            }

        },
        // internal error in the code executed inside the thread
        Ok(Err(e)) => Err(NotFound(e.to_string())),
        // any thread related error
        _ => Err(NotFound(Status::NotFound.to_string())),
    }
    
}

/// # git init a repo for testing only
#[openapi(tag = "Pull Requests")]
#[get("/init/<repo>")]
pub async fn init_repo(repo: &str) -> Result<String, NotFound<String>> {
    let repo_name_clone = repo.to_string(); // Clone the string

    let result = spawn_blocking(move || {
        let base_repo_path = PathHandler::get_relative_path("");
        let result = git_commands::Init::new().execute(Some(vec![&repo_name_clone]))
            .map(|_| "Repository initialized successfully".to_string())
            .map_err(|e| e.to_string());
        PathHandler::set_relative_path(&base_repo_path);
        result
    }).await;

    match result {
        // all good
        Ok(Ok(message)) => Ok(message),
        // internal error in the code executed inside the thread
        Ok(Err(e)) => Err(NotFound(e.to_string())),
        // any thread related error
        _ => Err(NotFound(Status::NotFound.to_string())),
    }
}

/// # Create a pull request
#[openapi(tag = "Pull Requests")]
#[post("/repos/<repo>/pulls", format = "application/json", data = "<pr>")]
pub async fn post_pull_request(state: &State<AppState>, repo: String, pr: Json<PullRequestBody>) -> Result<String, NotFound<String>> {
    // 1. Extract data from the Json<PullRequestBody> parameter
    let pull_request_data = pr.into_inner();

    let pull_request_resource = match PullRequest::new(pull_request_data, repo) {
        Ok(resource) => resource,
        Err(e) => {
            // Handle the error
            println!("Error creating pull request resource: {}", e);
            // Optionally, return or propagate the error
            return Err(NotFound(e.to_string())); // Assuming you are in a function that returns Result
        }
    };

    match create(&pull_request_resource, &state.db_pool).await {
        Ok(pull_request_id) => {
            Ok(format!("Pull request created successfully with Id: {}", pull_request_id))
        }
        Err(err) => {
            Err(NotFound(format!("Error creating pull request: {:?}", err)))
        }
    }
}


