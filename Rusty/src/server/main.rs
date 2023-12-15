#[macro_use]
extern crate rocket;
use rusty::commands::git_commands::RELATIVE_PATH;
//use rusty::server::server_protocol::ServerProtocol;
use std::env;
use rocket::tokio::task::spawn_blocking;
//use std::thread;


use rusty::commands::git_commands;
use rusty::commands::git_commands::Command;

#[get("/repo/<repo_name>")]
async fn world(repo_name: &str) -> String {
    let _vec = spawn_blocking(|| {
        if let Err(e) = git_commands::Init::new().execute(None) {
            println!("e {}",e);
        }
    }).await;
    format!("repo_name {}", repo_name)
}

#[tokio::main]
async fn main() {
    env::set_var(RELATIVE_PATH, "src/server/");
    // Spawn a new Tokio task to run the Rocket application
    tokio::spawn(async {
        if let Err(e) = run_rocket().await {
            eprintln!("Rocket error: {}", e);
        }
    });

    // Your main application logic can continue here if needed
    println!("MAIN THREAD CONTINUED. SLEEP 1000 AND SHUTDOWN");
    tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
}

async fn run_rocket() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![world])
        .launch()
        .await?;
    Ok(())
}
