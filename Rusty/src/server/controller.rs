use crate::server::server_protocol::ServerProtocol;
use crate::commands::git_commands::RELATIVE_PATH;
use std::env;
use rocket::tokio::task::spawn_blocking;
use std::thread;
use rocket::{get, routes};
use crate::commands::git_commands;
use crate::commands::git_commands::Command;


#[get("/repo/<repo_name>")]
pub async fn world(repo_name: &str) -> String {
    let _vec = spawn_blocking(|| {
        if let Err(e) = git_commands::Init::new().execute(None) {
            println!("e {}",e);
        }
    }).await;
    format!("repo_name {}", repo_name)
}