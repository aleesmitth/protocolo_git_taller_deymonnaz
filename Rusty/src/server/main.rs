use rusty::commands::commands::RELATIVE_PATH;
use rusty::server::server_protocol::ServerProtocol;
use std::env;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var(RELATIVE_PATH, "src/server/");
    println!("Git server is running at git://127.0.0.1:9418");
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
