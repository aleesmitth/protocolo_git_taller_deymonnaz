use chrono;
use sqlx::PgPool;
use std::error::Error;
use std::io;
use rocket::response::status::NotFound;
use sqlx::Row;
use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use crate::commands::helpers::check_if_repo_exists;

// TODO make a constructor for this, so it validates data or whatever, is it cleaner?
// TODO we should use traits to support more than 1 model.
// TODO refactor to use macros so that queries are verified at compile time

pub struct AppState {
    pub db_pool: PgPool,
}

///
/// Struct used to retrieve a PullRequest from the database
///
#[derive(Debug, Default)]
pub struct PullRequestOptions {
    pub _id: Option<i32>,
    pub repo: Option<String>,
    pub head: Option<String>,
    pub base: Option<String>,
    pub commit_after_merge: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>
}

///
/// main PullRequest resource
///
#[derive(Debug,FromRow,Serialize,Deserialize, JsonSchema)]
pub struct PullRequest {
    pub _id: Option<i32>,
    pub repo: String,
    pub head: String,
    pub base: String,
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
    pub head: String,
    pub base: String,
}

fn sample_commit() -> &'static str {
    "ec2b86e15c8deec7b041e622bca5cd9f258888c9"
}
fn sample_date() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

impl PullRequest {
    pub fn new(request_body: PullRequestBody, repo: String) -> Result<Self, Box<dyn Error>> {
        if request_body.base.is_empty() || request_body.head.is_empty() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Pull request body has to have populated base and head in request body",
            )))
        }
        if request_body.head == request_body.base {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Can't create a PullRequest with head same as base",
            )))
        }

        if let Err(_) =  check_if_repo_exists(repo.as_str()) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error repo does not exist, pull request creation aborted.",
            )))
        }
        //TODO check if head and base are inside repo
        Ok(PullRequest {
            _id: None,
            repo,
            head: request_body.head,
            base: request_body.base,
            commit_after_merge: None,
            created_at: None
        })
    }
}
