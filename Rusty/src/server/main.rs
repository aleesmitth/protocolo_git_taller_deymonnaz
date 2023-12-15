#[macro_use]
extern crate rocket;
use rusty::commands::git_commands::RELATIVE_PATH;
use rusty::server::server_protocol::ServerProtocol;
use std::env;
use rocket::tokio::task::spawn_blocking;
use std::thread;


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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var(RELATIVE_PATH, "src/server/");

    // Spawn a new Tokio task to run the Rocket application

    tokio::spawn(async {
        if let Err(e) = run_rocket().await {
            eprintln!("Rocket error: {}", e);
        }
    });

    let listener = ServerProtocol::bind("127.0.0.1:9418")?; // Default Git port
    println!("bind complete");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut cloned_stream = stream.try_clone()?;
                thread::spawn(move || {
                    if let Err(err) = ServerProtocol::handle_client_conection(&mut cloned_stream) {
                        println!("Error: {:?}", err);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    // Your main application logic can continue here if needed
    println!("MAIN THREAD CONTINUED. SLEEP 1000 AND SHUTDOWN");
    tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
    Ok(())
}

async fn run_rocket() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![world])
        .launch()
        .await?;
    Ok(())
}
