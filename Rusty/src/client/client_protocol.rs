use crate::commands::helpers;
use std::sync::mpsc;
use crate::commands::protocol_utils;

use std::{
    error::Error, fs, io::BufRead, io::Read, io::Write, net::Shutdown, net::TcpStream,
    str, thread, time::Duration,
};
pub struct ClientProtocol;

impl ClientProtocol {
    pub fn new() -> Self {
        ClientProtocol {}
    }

    pub fn connect(address: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
        println!("connecting to server...");
        //let remote_server_address = helpers::get_remote_url(DEFAULT_REMOTE)?;
        Ok(TcpStream::connect(address)?)
    }

    pub fn receive_pack(&mut self) -> Result<(), Box<dyn Error>> {
        let mut stream = ClientProtocol::connect("127.0.0.1:9418")?;
        println!("connect complete");

        let service = "git-receive-pack /projects/.git\0host=127.0.0.1\0";
        let request = format!("{:04x}{}", service.len() + 4, service);
        println!("request {}", request);
        // Send the Git service request
        stream.write_all(request.as_bytes())?;
        let _ = stream.flush();
        println!("request sent");
        // Read the response from the server
        let mut response = String::new();

        let reader = std::io::BufReader::new(&stream);
        let mut remote_hash = String::new();
        for line in reader.lines() {
            if let Ok(value) = line {
                let split_value: Vec<&str> = value.split_whitespace().collect();
                remote_hash = split_value[0].to_string()[4..].to_string();
                println!("response line: {:?}", value);
                println!("remote hash: {}", remote_hash);
            }
            break;
        }

        response.clear();
        let branch_path = helpers::get_current_branch_path()?;
        let last_commit_hash: String = helpers::read_file_content(&branch_path)?;
        println!("last_commit: {}", last_commit_hash);
        let line = format!("{} {} refs/heads/main", remote_hash, last_commit_hash);
        let actual_line = format!("{:04x}{}\n", line.len() + 5, line);
        println!("push line: {}", actual_line);
        stream.write_all(actual_line.as_bytes())?;
        stream.write_all(b"0000")?;
        stream.flush()?;

        let mut pack_file = fs::File::open(".git/pack/pack_file.pack")?;
        std::io::copy(&mut pack_file, &mut stream)?;
        // stream.flush()?;

        let reader = std::io::BufReader::new(&stream);
        for line in reader.lines() {
            let line = line?;
            println!("response line: {}", line);
            break;
        }

        Ok(())
    }

    pub fn fetch_from_remote_with_our_server(
        &mut self,
        remote_url: String,
    ) -> Result<Vec<(String, String)>, Box<dyn Error>> {
        let mut stream = ClientProtocol::connect(&remote_url)?;

        // TODO a nuestro server no le importa el relative path por ahora, que pasa si tenemos mas de 1 repo en el sv? arreglar.
        let request = protocol_utils::format_line_to_send("git-upload-pack /projects/.git\0host=127.0.0.1\0".to_string());
        println!("{}", request);

        stream.write_all(request.as_bytes())?;

        stream.flush()?;
        println!("request enviada");

        let mut refs_in_remote: Vec<(String, String)> = Vec::new();
        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        let response_received: Vec<String> =
            protocol_utils::read_until(&mut reader, protocol_utils::REQUEST_DELIMITER_DONE, true)?;

        println!("response received {:?}", response_received);
        for line in response_received {
            if let [remote_hash, branch_name, ..] = line.split_whitespace().collect::<Vec<&str>>().as_slice() {
                refs_in_remote.push((remote_hash.to_string(), branch_name.to_string()));
            }
        }
        println!("branches received");
        //return Ok(Vec::new());
        for (ref_hash, _ref_name) in &refs_in_remote {
            let want_request = protocol_utils::format_line_to_send(format!("{} {}\n", protocol_utils::WANT_REQUEST, ref_hash));
            println!("want_request sent: {}", want_request.clone());
            stream.write_all(want_request.as_bytes())?;
            break;
        }
        let _ = stream.write_all(protocol_utils::REQUEST_LENGTH_CERO.as_bytes());
        println!("sent 0000");
        let _ = stream.write_all(protocol_utils::format_line_to_send(protocol_utils::REQUEST_DELIMITER_DONE.to_string()).as_bytes());
        println!("sent done");
        stream.flush()?;

        let _: Vec<String> =
            protocol_utils::read_until(&mut reader, protocol_utils::NAK_RESPONSE, false)?;
        println!("received NAK");
        let _ = stream.write_all(protocol_utils::format_line_to_send(protocol_utils::REQUEST_DELIMITER_DONE.to_string()).as_bytes());
        println!("sent done");
        // TODO RECEIVE PACKFILE
        println!("TODO RECEIVE PACKFILE");
                        /*
                        stream.read_to_end(&mut buffer)?;
                        println!("{:?}", buffer);
                        // std::fs::write(".git/pack/received_pack_file.pack", &buffer)?;
                        let mut file = fs::File::create(".git/pack/received_pack_file.pack")?;
                        file.write_all(&buffer)?;
                        */

        stream.shutdown(Shutdown::Both)?;

        Ok(refs_in_remote)
    }

//TODO MATAR ESTA FUNCION
    pub fn fetch_from_remote(
        &mut self,
        remote_url: String,
    ) -> Result<Vec<(String, String)>, Box<dyn Error>> {
        let mut stream = ClientProtocol::connect(&remote_url)?;

        let request = format!(
            "{:04x}git-upload-pack /projects/.git\0host=127.0.0.1\0",
            "git-upload-pack /projects/.git\0host=127.0.0.1\0".len() + 4
        );
        println!("{}", request);
        stream.write_all(request.as_bytes())?;
        stream.flush()?;

        let (tx, rx) = mpsc::channel();
        let stream_clone = stream.try_clone()?;
        // Spawn a new thread to handle reading from the server
        let thread_handle = thread::spawn(move || {
            let _ = ClientProtocol::read_response_from_server(stream_clone, tx);
        });

        let mut refs_in_remote: Vec<(String, String)> = Vec::new();
        let mut current_commit ;
        thread::sleep(Duration::from_millis(10));
        loop {
            match rx.try_recv() {
                Ok(message) => {
                    if message == "ReadingDone" {
                        break;
                    }
                    let split_value: Vec<&str> = message.split_whitespace().collect();
                    let remote_hash = split_value[0].to_string()[4..].to_string();
                    println!("split value: {:?}", split_value);
                    println!("response line: {:?}\n", message);
                    println!("remote hash: {}", remote_hash);
                    if split_value.len() > 2 {
                        current_commit = remote_hash;
                    } else {
                        //aca tengo que hacer que se guarde el commit y nombre de la ref
                        let ref_name = split_value[1];
                        refs_in_remote.push((remote_hash, ref_name.to_string()));
                    }
                }
                Err(_) => break,
            }
            thread::sleep(Duration::from_millis(10));
        }

        println!("\nafter response");
        println!("{:?}", refs_in_remote);
        for (ref_hash, _ref_name) in &refs_in_remote {
            let line = format!("want {}\n", ref_hash);
            let actual_line = format!("{:04x}{}", line.len() + 4, line);
            println!("request line: {}", actual_line);
            stream.write_all(actual_line.as_bytes())?;
            // TODO remove this extra writes, it's for testing
            stream.write_all(actual_line.as_bytes())?;
            stream.write_all(actual_line.as_bytes())?;
            break;
        }
        stream.write_all("0000".as_bytes())?;
        let done = format!("{:04x}done\n", "done\n".len() + 4);
        println!("{}", done);
        stream.write_all(done.as_bytes())?;
        stream.flush()?;

        thread::sleep(Duration::from_millis(100));
        loop {
            match rx.try_recv() {
                Ok(message) => {
                    if message == "00000008NAK" {
                        let mut buffer = Vec::new();
                        stream.read_to_end(&mut buffer)?;
                        println!("{:?}", buffer);
                        // std::fs::write(".git/pack/received_pack_file.pack", &buffer)?;
                        let mut file = fs::File::create(".git/pack/received_pack_file.pack")?;
                        file.write_all(&buffer)?;
                    }
                }
                Err(_) => break,
            }
            thread::sleep(Duration::from_millis(10));
        }

        thread_handle.join().expect("Error joining thread");
        stream.shutdown(Shutdown::Both)?;

        Ok(refs_in_remote)
    }

//TODO MATAR ESTA FUNCION
    fn read_response_from_server(
        stream: TcpStream,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn Error>> {
        let reader = std::io::BufReader::new(&stream);

        for line in reader.lines() {
            if let Ok(value) = line {
                // Send the received line to the main thread
                println!("line thread: {}", value);
                tx.send(value.clone())?;
                if value == "00000008NAK" {
                    break;
                }
            }
        }

        // Send a signal to the main thread that reading is done
        tx.send("ReadingDone".to_string())?;
        Ok(())
    }

    /* fn write_lines_to_stream(
        stream: &mut TcpStream,
        lines: Vec<String>,
    ) -> Result<(), Box<dyn Error + '_>> {
        for line in lines {
            stream.write_all(line.as_bytes())?;
        }
        stream.flush()?;
        Ok(())
    } */
}
