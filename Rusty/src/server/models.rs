use chrono;
use sqlx::PgPool;
use std::error::Error;
use std::io;
use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use crate::commands::git_commands::PathHandler;
use crate::commands::helpers::{check_if_branch_belongs_to_repo, check_if_repo_exists};

// TODO make a constructor for this, so it validates data or whatever, is it cleaner?
// TODO we should use traits to support more than 1 model.
// TODO refactor to use macros so that queries are verified at compile time

pub struct AppState {
    pub db_pool: PgPool,
    pub relative_path_handler: PathHandler,
}

///
/// Struct used to retrieve a PullRequest from the database
///
#[derive(Debug, Default)]
pub struct PullRequestOptions {
    pub _id: Option<i32>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub repo: Option<String>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub commit_after_merge: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>
}

///
/// main PullRequest resource
///
#[derive(Debug,FromRow,Serialize,Deserialize, JsonSchema, Clone)]
pub struct PullRequest {
    pub _id: Option<i32>,
    pub title: String,
    pub body: Option<String>,
    pub repo: String,
    pub base: String,
    pub head: String,
    #[schemars(example = "sample_commit")]
    pub commit_after_merge: Option<String>,
    #[schemars(example = "sample_date")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>
}

///
/// Struct used to receive requests with body from HTTP
///
#[derive(Debug,FromRow,Serialize,Deserialize, JsonSchema)]
pub struct PullRequestBody {
    pub title: String,
    pub body: Option<String>,
    pub base: String,
    pub head: String,
}

fn sample_commit() -> &'static str {
    "ec2b86e15c8deec7b041e622bca5cd9f258888c9"
}
fn sample_date() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

impl PullRequest {
    pub fn new(request_body: PullRequestBody, repo: String, path_handler: &mut PathHandler) -> Result<Self, Box<dyn Error>> {
        // TODO handle errors more elegantly
        if request_body.title.is_empty() || request_body.base.is_empty() || request_body.head.is_empty() || repo.is_empty() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Pull request body has to have populated title, base, head, and repo in request",
            )))
        }
        if request_body.head == request_body.base {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Can't create a PullRequest with head same as base",
            )))
        }

        check_if_repo_exists(repo.as_str(), path_handler)?;
        path_handler.set_relative_path(path_handler.get_relative_path(&repo));
        check_if_branch_belongs_to_repo(request_body.base.as_str(), repo.as_str(), path_handler)?;
        check_if_branch_belongs_to_repo(request_body.head.as_str(), repo.as_str(), path_handler)?;

        Ok(PullRequest {
            _id: None,
            title: request_body.title,
            body: request_body.body,
            repo,
            base: request_body.base,
            head: request_body.head,
            commit_after_merge: None,
            created_at: None
        })
    }
}
