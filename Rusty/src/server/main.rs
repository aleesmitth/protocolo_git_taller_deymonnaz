#[macro_use]
extern crate rocket;
use rusty::commands::git_commands::RELATIVE_PATH;
//use rusty::server::server_protocol::ServerProtocol;
use rusty::server::controller::*;
use std::env;
//use rocket::tokio::task::spawn_blocking;
//use std::thread;

use dotenv::dotenv;
use sqlx::Connection;
use rusty::server::models::PullRequest;
use rusty::server::models;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var(RELATIVE_PATH, "src/server/");

    //TODO MOVE THIS TO DATABASE SCRIPT
    dotenv().ok();
    // Command to run the shell script
    let script_path = "create_database.sh";

    // Execute the shell script to create the database if it doesn't exist
    let status = Command::new("sh")
        .arg(script_path)
        .status()
        .expect("Failed to execute the script");

    if status.success() {
        println!("Script executed successfully");
    } else {
        println!("Script execution failed");
    }

    let url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    println!("url: {}", url);
    let pool = sqlx::postgres::PgPool::connect(&url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("migrated");
    let pr = PullRequest {
        id: None,
        name: "hello_world_branch".to_string()
    };

    models::create(&pr, &pool).await?;
    // TODO
    /*dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    println!("database_url: {:?}", database_url);
    let connection = PgConnection::establish(&database_url);
    match connection {
        Ok(_) => println!("Connected to the database"),
        Err(err) => eprintln!("Error connecting to the database: {}", err),
    }*/


    // Spawn a new Tokio task to run the Rocket application

    /*tokio::spawn(async {
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
*/
    Ok(())
}

async fn run_rocket() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![world])
        .launch()
        .await?;
    Ok(())
}
