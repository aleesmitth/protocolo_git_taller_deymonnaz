#[macro_use]
extern crate rocket;
use rusty::commands::git_commands::RELATIVE_PATH;
//use rusty::server::server_protocol::ServerProtocol;
use rusty::server::controller::*;
use rusty::server::database::Database;
use rusty::server::server_protocol::ServerProtocol;
use std::{env, thread};
//use rocket::tokio::task::spawn_blocking;
//use std::thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var(RELATIVE_PATH, "src/server/");

    let database = Database::new();

    database.run().await?;

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

    Ok(())
}

async fn run_rocket() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![world, init_repo])
        .launch()
        .await?;
    Ok(())
}
