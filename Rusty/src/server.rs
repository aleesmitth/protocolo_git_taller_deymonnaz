use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_git_client(mut stream: TcpStream) {
    // In the Git Dumb Protocol, Git commands are sent as text lines.
    // You would parse the incoming lines and respond accordingly.

    let reader = std::io::BufReader::new(&stream);
    let mut writer = std::io::BufWriter::new(&stream);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("git-upload-pack") {
                writer.write_all(b"# service=git-upload-pack\n").unwrap();
                writer.flush().unwrap();
            } else if line.starts_with("git-receive-pack") {
                writer.write_all(b"# service=git-receive-pack\n").unwrap();
                writer.flush().unwrap();
            } else if line == "capabilities" {
                writer.write_all(b"delete-refs side-band-64k\n").unwrap();
                writer.write_all(b"multi_ack\n").unwrap();
                writer.write_all(b"side-band\n").unwrap();
                writer.write_all(b"ofs-delta\n").unwrap();
                writer.write_all(b"thin-pack\n").unwrap();
                writer.write_all(b"shallow\n").unwrap();
                writer.write_all(b"no-progress\n").unwrap();
                writer.write_all(b"include-tag\n").unwrap();
                writer.write_all(b"multi_ack_detailed\n").unwrap();
                writer.flush().unwrap();
            }
        } else {
            break;
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9418").unwrap(); // Default Git port

    println!("Git server (Dumb Protocol) is running at git://127.0.0.1:9418");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_git_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}