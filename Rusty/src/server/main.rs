#[macro_use]
extern crate rocket;
use rusty::{commands::git_commands::RELATIVE_PATH, server::models::AppState};
//use rusty::server::server_protocol::ServerProtocol;
use rusty::server::controller::*;
use rusty::server::database::Database;
use rusty::server::server_protocol::ServerProtocol;
use std::{env, thread};
//use rocket::tokio::task::spawn_blocking;
//use std::thread;
use rocket::State;
use sqlx::PgPool;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var(RELATIVE_PATH, "src/server/");

    let database = Database::new();

    let db_pool = database.run().await?;

    // Spawn a new Tokio task to run the Rocket application
    tokio::spawn(async {
        if let Err(e) = run_rocket(db_pool).await {
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

    Ok(())
}

async fn run_rocket(db_pool: PgPool) -> Result<(), rocket::Error> {
    let state = AppState { db_pool };

    let _rocket = rocket::build()
        .manage(state)
        .mount("/", routes![get_pull_request, init_repo, new_pr]);

    // If you don't need to customize the Rocket configuration, you can use the default configuration.
    let _rocket = _rocket.launch().await;
    println!("rocket:{:?}", _rocket);
    Ok(())
}

