extern crate rocket;
use rusty::{constants::RELATIVE_PATH, server::models::AppState};
use std::collections::HashSet;
use std::sync::{Mutex, Arc, Condvar};
//use rusty::server::server_protocol::ServerProtocol;
use rusty::server::controller::*;
use rusty::server::database::Database;
use rusty::server::server_protocol::ServerProtocol;
use std::{env, thread};
//use rocket::tokio::task::spawn_blocking;
//use std::thread;
use sqlx::PgPool;

use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi_get_routes, rapidoc::*, swagger_ui::*};



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

    // Create a HashSet to store locked branch names
    let locked_branches = Arc::new((Mutex::new(HashSet::new()), Condvar::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cloned_locked_branches = Arc::clone(&locked_branches);
                let mut cloned_stream = stream.try_clone()?;
                thread::spawn(move || {
                    if let Err(err) = ServerProtocol::handle_client_conection(&mut cloned_stream, cloned_locked_branches) {
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

    /*let _rocket = rocket::build()
        .manage(state)
        .mount("/", routes![
            test_api,
            get_pull_request_commits,
            get_repo_pull_request,
            get_pull_request,
            init_repo,
            post_pull_request,
            put_merge
        ]);*/

    let _rocket = rocket::build()
        .manage(state)
        .mount(
            "/",
            openapi_get_routes![
                init_repo,
                post_pull_request,
                get_repo_pull_request,
                get_pull_request,
                put_merge,
                get_pull_request_commits,
                test_api,
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );


    // If you don't need to customize the Rocket configuration, you can use the default configuration.
    let _rocket = _rocket.launch().await;
    println!("rocket:{:?}", _rocket);
    Ok(())
}

