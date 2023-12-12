use rusty::commands::commands::Command;
use rusty::commands::commands::Init;
use rusty::commands::commands::Commit;
use rusty::commands::commands::Add;
use rusty::commands::commands::RELATIVE_PATH;
use rusty::server::server_protocol::ServerProtocol;
use std::env;
use std::io::prelude::*;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var(RELATIVE_PATH, "src/server/");
    println!("Git server is running at git://127.0.0.1:9418");
    let listener = ServerProtocol::bind("127.0.0.1:9418")?; // Default Git port
    println!("bind complete");

    // Init::new().execute(None)?;
    // Add::new().execute(Some(vec!["file.txt"]))?;
    // Commit::new().execute(Some(vec!["-m", "test"]))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut cloned_stream = stream.try_clone()?;
                thread::spawn(move || {
                    ServerProtocol::handle_client_conection(&mut cloned_stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
    Ok(())
}
