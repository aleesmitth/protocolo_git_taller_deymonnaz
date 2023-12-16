use rocket::tokio::task::spawn_blocking;
use rocket::{get, routes};
use crate::commands::git_commands;
use crate::commands::git_commands::Command;


#[get("/repo/<repo_name>")]
pub async fn world(repo_name: &str) -> String {
    let repo_name_clone = repo_name.to_string(); // Clone the string
    let _vec = spawn_blocking(move || {
        if let Err(e) = git_commands::Init::new().execute(Some(vec![&repo_name_clone])) {
            println!("e {}",e);
        }
    }).await;
    format!("repo name: {}", repo_name)
}