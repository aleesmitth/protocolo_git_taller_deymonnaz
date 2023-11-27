use std::io::prelude::*;
use std::thread;
use rusty::commands::commands::Command;
use rusty::commands::commands::Init;
use rusty::commands::commands::RELATIVE_PATH;
use rusty::commands::structs::Head;
use std::env;
use rusty::server::server_protocol::ServerProtocol;


fn main() -> Result<(), Box<dyn std::error::Error>> {    
    env::set_var(RELATIVE_PATH, "src/server/");
    println!("Git server is running at git://127.0.0.1:9418");
    let listener = ServerProtocol::bind("127.0.0.1:9418")?; // Default Git port
    println!("bind complete");

    // let mut head = Head::new();
    // Init::new().execute(&mut head, None)?;

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