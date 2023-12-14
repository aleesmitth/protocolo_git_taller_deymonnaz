#[macro_use]
extern crate rocket;

#[get("/")]
fn world() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() {
    // Spawn a new Tokio task to run the Rocket application
    tokio::spawn(async {
        if let Err(e) = run_rocket().await {
            eprintln!("Rocket error: {}", e);
        }
    });

    // Your main application logic can continue here if needed

    println!("MAIN THREAD CONTINUED. SLEEP 10 AND SHUTDOWN");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
}

async fn run_rocket() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![world])
        .launch()
        .await?;
    Ok(())
}
