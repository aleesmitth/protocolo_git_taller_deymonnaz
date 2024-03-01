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
use git_commands::Log;
use crate::commands::helpers::{get_branch_last_commit, get_branch_path};


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
    let options = PullRequestOptions { repo: Some(repo), ..Default::default() };

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
    let options = PullRequestOptions { repo: Some(repo.clone()), _id: Some(pull_number), ..Default::default() };

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
pub async fn get_pull_request_commits(state: &State<AppState>, repo: String, pull_number: i32) -> Result<String, NotFound<String>> {
    let options = PullRequestOptions { repo: Some(repo.clone()), _id: Some(pull_number), ..Default::default() };

    match read(&options, &state.db_pool).await {
        Ok(pull_requests) => {
            // only one pull request should've been returned
            // assuming that only the first pull request is the valid one
            let head = pull_requests[0].head.clone();
            let base = pull_requests[0].base.clone();
            let commit_after_merge = pull_requests[0].commit_after_merge.clone();
            let base_repo_path = PathHandler::get_relative_path("");
            PathHandler::set_relative_path(&format!("{}{}/", base_repo_path, repo));
            if let Some(commit) = commit_after_merge {
                let merge_log = Log::new().execute(Some(vec![&commit]));
                PathHandler::set_relative_path(&base_repo_path);
                match merge_log {
                    Ok(log) => Ok(format!("-- PullRequest Merge Log --\n{}\n", log)),
                    Err(err) => Err(NotFound(format!("Error fetching merge commit log: {:?}", err)))
                }
            } else {
                let head_last_commit = match get_branch_last_commit(&get_branch_path(&head)) {
                    Ok(commit) => commit,
                    Err(e) => {
                        PathHandler::set_relative_path(&base_repo_path);
                        return Err(NotFound(format!("Error finding last commit of branch {}, {}", head, e)));
                    }
                };
                let base_last_commit = match get_branch_last_commit(&get_branch_path(&base)) {
                    Ok(commit) => commit,
                    Err(e) => {
                        PathHandler::set_relative_path(&base_repo_path);
                        return Err(NotFound(format!("Error finding last commit of branch {}, {}", base, e)));
                    }
                };
                let logs_head = match Log::new().execute(Some(vec![&head_last_commit])) {
                    Ok(log) => format!("-- PullRequest Head Log --\n{}\n", log),
                    Err(err) => {
                        PathHandler::set_relative_path(&base_repo_path);
                        return Err(NotFound(format!("Error fetching PullRequest head log: {:?}", err)))
                    }
                };
                let logs_base = match Log::new().execute(Some(vec![&base_last_commit])) {
                    Ok(log) => format!("-- PullRequest Base Log --\n{}\n", log),
                    Err(err) => {
                        PathHandler::set_relative_path(&base_repo_path);
                        return Err(NotFound(format!("Error fetching PullRequest head log: {:?}", err)))
                    }
                };

                PathHandler::set_relative_path(&base_repo_path);
                Ok(format!("{}{}", logs_head, logs_base))
            }
        }
        Err(err) => Err(NotFound(format!("Error fetching pull request: {:?}", err)))
    }
}

/// # Merge a pull request
///
// #[get("/repos/<repo>/pulls/<pull_number>/merge", format = "application/json", data = "<pr>")]
/// Merges a pull request into the base branch.
#[openapi(tag = "Pull Requests")]
#[put("/repos/<repo>/pulls/<pull_number>")]
pub async fn put_merge(state: &State<AppState>, repo: String, pull_number: i32) -> Result<String, NotFound<String>> {
    let options = PullRequestOptions { repo: Some(repo.clone()), _id: Some(pull_number), ..Default::default() };

    let pull_requests = match read(&options, &state.db_pool).await {
        Ok(pull_requests) => pull_requests,
        Err(err) => return Err(NotFound(err.to_string()))
    };
    let mut merged_pull_request = pull_requests[0].clone();
    println!("pull request: {:?}", pull_requests);
    if pull_requests.is_empty(){
        return Err(NotFound("Pull request came back empty".to_string()))
    }
    if pull_requests[0].commit_after_merge.is_some(){
        return Err(NotFound("Pull request already merged".to_string()))
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
                Err(_) => Err(NotFound("Error updating the PR in the database after merge".to_string()))
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


