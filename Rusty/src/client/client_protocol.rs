use crate::commands::protocol_utils;
use crate::commands::structs::Head;
use crate::commands::git_commands::{Command, PackObjects, PathHandler};
use crate::constants::{ZERO_HASH, REQUEST_DELIMITER_DONE, REQUEST_LENGTH_CERO, WANT_REQUEST, NAK_RESPONSE};

use std::{
    error::Error, fs, io::Read, io::Write, net::Shutdown, net::TcpStream, str, thread,
    time::Duration,
};
use crate::commands::helpers::get_client_current_working_repo;

pub struct ClientProtocol;

impl Default for ClientProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientProtocol {
    pub fn new() -> Self {
        ClientProtocol {}
    }

    pub fn connect(address: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
        println!("connecting to server...");
        Ok(TcpStream::connect(address)?)
    }

    pub fn receive_pack(&mut self, remote_url: String) -> Result<(), Box<dyn Error>> {
        let mut stream = ClientProtocol::connect(&remote_url)?;
        // println!("connect complete");
        let current_repo = get_client_current_working_repo()?;
        let request = protocol_utils::format_line_to_send(
            format!("git-receive-pack /{}/.git\0host=127.0.0.1\0", current_repo).to_string(),
        );
        // println!("request {}", request);

        stream.write_all(request.as_bytes())?;

        stream.flush()?;

        // println!("request sent");
        // Read the response from the server
        let mut refs_in_remote: Vec<(String, String)> = Vec::new();
        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        let response_received: Vec<String> =
            protocol_utils::read_until(&mut reader, REQUEST_DELIMITER_DONE, true)?;

        // println!("response received {:?}", response_received);
        for line in response_received.clone() {
            if let [remote_hash, branch_name, ..] =
                line.split_whitespace().collect::<Vec<&str>>().as_slice()
            {
                refs_in_remote.push((remote_hash.to_string(), branch_name.to_string()));
            }
        }

        let current_branch_ref = Head::get_current_branch_ref()?;
        let last_commit_hash: String = Head::get_head_commit()?.replace('\n', "");
        // println!("last_commit: {}", last_commit_hash);
        // println!("refs in remote: {:?}", refs_in_remote);
        let mut push_line = String::new();
        for (ref_hash, ref_name) in &refs_in_remote {
            // let want_request = protocol_utils::format_line_to_send(format!("{} {}\n", protocol_utils::WANT_REQUEST, ref_hash));
            // println!("want_request sent: {}", want_request.clone());
            // println!("{} == {}", ref_name, current_branch_ref);
            if *ref_name == current_branch_ref {
                push_line = protocol_utils::format_line_to_send(format!(
                    "{} {} {}\n",
                    ref_hash, last_commit_hash, ref_name
                ));
                // println!("push line: {}", push_line);
            }
        }

        if push_line.is_empty() {
            push_line = protocol_utils::format_line_to_send(format!(
                "{} {} {}\n",
                ZERO_HASH, last_commit_hash, current_branch_ref
            ));
            // println!("push line: {}", push_line);
        }

        stream.write_all(push_line.as_bytes())?;

        let _ = stream.write_all(REQUEST_LENGTH_CERO.as_bytes());
        // println!("sent 0000");

        let pack_checksum = PackObjects::new().execute(Some(vec![&last_commit_hash]))?;
        let pack_file_path = format!(".git/pack/pack-{}.pack", pack_checksum);
        // println!("{}", pack_file_path);
        let mut pack_file = fs::File::open(pack_file_path)?;
        let mut buffer = Vec::new();
        pack_file.read_to_end(&mut buffer)?;
        // println!("buffer: {:?}", buffer);

        thread::sleep(Duration::from_millis(500));
        stream.write_all(&buffer)?;

        // println!("sending pack file");
        stream.flush()?;

        stream.shutdown(Shutdown::Write)?;

        Ok(())
    }

    pub fn fetch_from_remote_with_our_server(
        &mut self,
        remote_url: String,
    ) -> Result<Vec<(String, String)>, Box<dyn Error>> {
        let mut stream = ClientProtocol::connect(&remote_url)?;

        let current_repo = get_client_current_working_repo()?;
        let request = protocol_utils::format_line_to_send(
            format!("git-upload-pack /{}/.git\0host=127.0.0.1\0", current_repo).to_string(),
        );
        // println!("{}", request);

        stream.write_all(request.as_bytes())?;

        stream.flush()?;
        // println!("request enviada");

        let mut refs_in_remote: Vec<(String, String)> = Vec::new();
        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        let response_received: Vec<String> =
            protocol_utils::read_until(&mut reader, REQUEST_DELIMITER_DONE, true)?;

        // println!("response received {:?}", response_received);
        for line in response_received {
            if let [remote_hash, branch_name, ..] =
                line.split_whitespace().collect::<Vec<&str>>().as_slice()
            {
                let split_branch_name: Vec<String> =
                    branch_name.split('\0').map(String::from).collect();
                refs_in_remote.push((remote_hash.to_string(), split_branch_name[0].to_string()));
            }
        }
        // println!("branches received");
        let mut _head_reference = String::new();
        let (first_ref_hash, first_ref_name) = &refs_in_remote[0];
        if first_ref_name.starts_with("HEAD") {
            _head_reference = first_ref_hash.clone();
            refs_in_remote.remove(0);
        }

        for (ref_hash, _ref_name) in &refs_in_remote {
            let want_request = protocol_utils::format_line_to_send(format!(
                "{} {}\n",
                WANT_REQUEST,
                ref_hash
            ));
            // println!("want_request sent: {}\nfor ref: {}", want_request.clone(), ref_name);
            stream.write_all(want_request.as_bytes())?;
        }
        let _ = stream.write_all(REQUEST_LENGTH_CERO.as_bytes());
        // println!("sent 0000");
        let _ = stream.write_all(
            protocol_utils::format_line_to_send(REQUEST_DELIMITER_DONE.to_string())
                .as_bytes(),
        );
        // println!("sent done");
        stream.flush()?;

        let _read_lines: Vec<String> =
            protocol_utils::read_until(&mut reader, NAK_RESPONSE, false)?;
        // println!("received NAK");

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        // println!("buffer received: {:?}", buffer);
        // ACA PODRIA HACER EL CHECKSUM: asi ya verifica apenas me llega que esta bien y sino lanzo error
        let mut file = fs::File::create(PathHandler::get_relative_path(
            ".git/pack/received_pack_file.pack",
        ))?;
        file.write_all(&buffer)?;

        // println!("pack file received");

        stream.shutdown(Shutdown::Both)?;

        Ok(refs_in_remote)
    }

    //TODO MATAR ESTA FUNCION
    //     pub fn fetch_from_remote(
    //         &mut self,
    //         remote_url: String,
    //     ) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    //         let mut stream = ClientProtocol::connect(&remote_url)?;

    //         let request = format!(
    //             "{:04x}git-upload-pack /projects/.git\0host=127.0.0.1\0",
    //             "git-upload-pack /projects/.git\0host=127.0.0.1\0".len() + 4
    //         );
    // println!("{}", request);
    //         stream.write_all(request.as_bytes())?;
    //         stream.flush()?;

    //         let (tx, rx) = mpsc::channel();
    //         let stream_clone = stream.try_clone()?;
    //         // Spawn a new thread to handle reading from the server
    //         let thread_handle = thread::spawn(move || {
    //             let _ = ClientProtocol::read_response_from_server(stream_clone, tx);
    //         });

    //         let mut refs_in_remote: Vec<(String, String)> = Vec::new();
    //         let mut current_commit ;
    //         thread::sleep(Duration::from_millis(10));
    //         loop {
    //             match rx.try_recv() {
    //                 Ok(message) => {
    //                     if message == "ReadingDone" {
    //                         break;
    //                     }
    //                     let split_value: Vec<&str> = message.split_whitespace().collect();
    //                     let remote_hash = split_value[0].to_string()[4..].to_string();
    //                     println!("split value: {:?}", split_value);
    //                     println!("response line: {:?}\n", message);
    //                     println!("remote hash: {}", remote_hash);
    //                     if split_value.len() > 2 {
    //                         current_commit = remote_hash;
    //                     } else {
    //                         //aca tengo que hacer que se guarde el commit y nombre de la ref
    //                         let ref_name = split_value[1];
    //                         refs_in_remote.push((remote_hash, ref_name.to_string()));
    //                     }
    //                 }
    //                 Err(_) => break,
    //             }
    //             thread::sleep(Duration::from_millis(10));
    //         }

    //         println!("\nafter response");
    //         println!("{:?}", refs_in_remote);
    //         for (ref_hash, _ref_name) in &refs_in_remote {
    //             let line = format!("want {}\n", ref_hash);
    //             let actual_line = format!("{:04x}{}", line.len() + 4, line);
    //             println!("request line: {}", actual_line);
    //             stream.write_all(actual_line.as_bytes())?;
    //             // TODO remove this extra writes, it's for testing
    //             stream.write_all(actual_line.as_bytes())?;
    //             stream.write_all(actual_line.as_bytes())?;
    //             break;
    //         }
    //         stream.write_all("0000".as_bytes())?;
    //         let done = format!("{:04x}done\n", "done\n".len() + 4);
    //         println!("{}", done);
    //         stream.write_all(done.as_bytes())?;
    //         stream.flush()?;

    //         thread::sleep(Duration::from_millis(100));
    //         loop {
    //             match rx.try_recv() {
    //                 Ok(message) => {
    //                     if message == "00000008NAK" {
    //                         let mut buffer = Vec::new();
    //                         stream.read_to_end(&mut buffer)?;
    //                         println!("{:?}", buffer);
    //                         // std::fs::write(".git/pack/received_pack_file.pack", &buffer)?;
    //                         let mut file = fs::File::create(".git/pack/received_pack_file.pack")?;
    //                         file.write_all(&buffer)?;
    //                     }
    //                 }
    //                 Err(_) => break,
    //             }
    //             thread::sleep(Duration::from_millis(10));
    //         }

    //         thread_handle.join().expect("Error joining thread");
    //         stream.shutdown(Shutdown::Both)?;

    //         Ok(refs_in_remote)
    //     }
}
