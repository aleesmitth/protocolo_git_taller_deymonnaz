use rocket::tokio::task::spawn_blocking;
use rocket::{get, post, put, routes};
use crate::commands::git_commands;
use crate::commands::git_commands::{Command, Pull};
use crate::server::models::PullRequest;
use rocket::serde::json::Json;
use serde::Deserialize;
use serde::Serialize;


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

#[post("/", format = "application/json", data = "<pr>")]
pub async fn new_pr(pr: Json<PullRequest>) -> String {
    println!("pr received in post: {:?}", pr);
    format!("shrug")
}
