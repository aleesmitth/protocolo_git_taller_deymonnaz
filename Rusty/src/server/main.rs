#[macro_use] extern crate rocket;

#[get("/")]
fn world() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![world])
        .launch()
        .await?;
    Ok(())
}