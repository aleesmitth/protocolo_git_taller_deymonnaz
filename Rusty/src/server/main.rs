use std::collections::HashSet;
use std::sync::{Mutex, Arc, Condvar};
use rusty::server::server_protocol::ServerProtocol;
use std::{env, thread};

use rusty::commands::git_commands::PathHandler;
use rusty::constants::SERVER_BASE_PATH;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //env::set_var(RELATIVE_PATH, "src/server/");

    /*let database = Database::new();

    let db_pool = database.run().await?;

    // Spawn a new Tokio task to run the Rocket application
    tokio::spawn(async {
        if let Err(e) = run_rocket(db_pool).await {
            eprintln!("Rocket error: {}", e);
        }
    });*/
    // Create a HashSet to store locked branch names
    let locked_branches = Arc::new((Mutex::new(HashSet::new()), Condvar::new()));

    let cloned_locked_branches_api = Arc::clone(&locked_branches);
    thread::spawn(move || {
        if let Err(err) = ServerProtocol::handle_api_requests(&cloned_locked_branches_api) {
            println!("Error: starting api handler thread {:?}", err);
        }
    });

    let listener = ServerProtocol::bind("127.0.0.1:9418")?; // Default Git port
    println!("bind complete");


    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cloned_locked_branches = Arc::clone(&locked_branches);
                let mut cloned_stream = stream.try_clone()?;
                let mut path_handler = PathHandler::new(SERVER_BASE_PATH.to_string());
                thread::spawn(move || {
                    if let Err(err) = ServerProtocol::handle_client_connection(&mut cloned_stream, &mut path_handler, cloned_locked_branches) {
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