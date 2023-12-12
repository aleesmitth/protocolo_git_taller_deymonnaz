use crate::commands::commands::PackObjects;
use crate::commands::commands::UnpackObjects;
use crate::commands::commands::Command;
use crate::commands::helpers;
use crate::commands::protocol_utils;
use crate::commands::structs::Head;
use crate::commands::commands::RELATIVE_PATH;
use crate::commands::commands::PathHandler;

use std::{
    error::Error, io, io::Write, net::TcpListener, net::TcpStream, io::Read, fs::File
};
const RECEIVE_PACK: &str = "git-receive-pack";
const UPLOAD_PACK: &str = "git-upload-pack";

pub struct ServerProtocol;

impl ServerProtocol {
    pub fn new() -> Self {
        ServerProtocol {}
    }

    pub fn bind(address: &str) -> Result<TcpListener, Box<dyn std::error::Error>> {
        println!("binding to client...");
        Ok(TcpListener::bind(address)?)
    }

    // fn read_message_to_buffer(reader: &mut dyn Read, message_length: u32) -> Result<[u8; message_length], Box<dyn std::error::Error>> {
    //     let mut buffer: [u8; message_length] = [0; message_length];
    //     if let Err(e) = reader.read_exact(&mut buffer) {
    //         return Err(Box::new(io::Error::new(
    //             io::ErrorKind::Other,
    //             "Error reading request",
    //         )));
    //     }
    //     Ok(buffer)
    // }

    pub fn handle_client_conection(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        // In the Git Dumb Protocol, Git commands are sent as text lines.
        // You would parse the incoming lines and respond accordingly.
        let mut stream_clone = stream.try_clone()?;
        let mut reader = std::io::BufReader::new(stream_clone);
        println!("waiting for request...");
        let request_length = protocol_utils::get_request_length(&mut reader)?;
        println!("request_length: {:?}", request_length);
        if request_length == 0 {
            // TODO gracefully end connection, message length is 0
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Request length 0",
            )));
        }
        //println!("{:x}", ServerProtocol::read_message_length(&mut reader).unwrap());
        // let mut writer = std::io::BufWriter::new(stream);
        println!("reading request_...");
        // let mut buffer = [0; 1024];
        // let _ = stream.read(&mut buffer);
        // println!("{:?}", buffer);
        let request = protocol_utils::read_exact_length_to_string(&mut reader, request_length)?;
        println!("request: {:?}", request);
        let request_array: Vec<&str> = request.split_whitespace().collect();
        println!("request in array: {:?}", request_array);
        match request_array[0] {
            UPLOAD_PACK => ServerProtocol::upload_pack(stream)?,
            RECEIVE_PACK => ServerProtocol::receive_pack(stream)?,
            _ => {}
        }

        println!("end handling connection");
        Ok(())
    }

    pub fn upload_pack(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        println!("git-upload-pack");
        let branches: Vec<String> = helpers::get_all_branches()?;
        for branch in &branches {
            let line_to_send = protocol_utils::format_line_to_send(branch.clone());
            println!("sending line: {:?}", line_to_send);
            stream.write_all(line_to_send.as_bytes())?;
        }

        let _ = stream.write_all(protocol_utils::REQUEST_LENGTH_CERO.as_bytes());
        println!("-sent end of message delimiter-");

        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        let requests_received: Vec<String> =
            protocol_utils::read_until(&mut reader, protocol_utils::REQUEST_DELIMITER_DONE, false)?;
        for request_received in requests_received.clone() {
            let request_array: Vec<&str> = request_received.split_whitespace().collect();
            println!("request in array: {:?}", request_array);
            if request_array[0] != protocol_utils::WANT_REQUEST {
                //TODO not want request, handle error gracefully
                println!(
                    "Error: expected want request but got: {:?}",
                    request_array[0]
                );
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Expecting want request but got something else",
                )));
            }
            // TODO should probably go read the branches again in case some other client updated the latest commit
            let is_valid_commit =
                ServerProtocol::validate_is_latest_commit_any_branch(request_array[1], &branches);
            if !is_valid_commit {
                println!("invalid commit: {:?}", request_array);
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Received invalid commit hash for want request",
                )));
            }

            println!("valid want request.");
        }

        let _ = stream.write_all(protocol_utils::format_line_to_send(protocol_utils::NAK_RESPONSE.to_string()).as_bytes());
        println!("sent NAK");
        // let _: Vec<String> =
        //     protocol_utils::read_until(&mut reader, protocol_utils::REQUEST_DELIMITER_DONE, false)?;
        println!("received done");
        println!("TODO SEND PACKFILE");
        let mut commits: Vec<String> = Vec::new();

        for request_received in requests_received.clone() {
            let request_array: Vec<&str> = request_received.split_whitespace().collect();
            if let Some(second_element) = request_array.get(1) {
                commits.push(second_element.to_string());
            }
        }

        let commits_str: Vec<&str> = commits.iter().map(|s| s.as_str()).collect();
        let checksum = PackObjects::new().execute(Some(commits_str.clone()))?;
        println!("created pack file");
        let pack_file_path = format!(".git/pack/pack-{}.pack", checksum);
        let mut pack_file = File::open(PathHandler::get_relative_path(&pack_file_path))?;
        let mut buffer = Vec::new();
        pack_file.read_to_end(&mut buffer);
        stream.write_all(&mut buffer);
        println!("sent pack file");

        Ok(())
    }

    pub fn validate_is_latest_commit_any_branch(commit: &str, branches: &Vec<String>) -> bool {
        for branch in branches {
            // Split the string into words
            let branch_commit_and_name: Vec<&str> = branch.split_whitespace().collect();
            if let Some(first_word) = branch_commit_and_name.first() {
                if first_word == &commit {
                    return true;
                }
            }
        }
        false
    }

    pub fn receive_pack(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        println!("git-receive-pack");

        let branches: Vec<String> = helpers::get_all_branches()?;
        for branch in &branches {
            let line_to_send = protocol_utils::format_line_to_send(branch.clone());
            println!("sending line: {:?}", line_to_send);
            stream.write_all(line_to_send.as_bytes())?;
        }

        let _ = stream.write_all(protocol_utils::REQUEST_LENGTH_CERO.as_bytes());

        let mut refs_to_update: Vec<(String, String, String)> = Vec::new();
        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        let requests_received: Vec<String> =
            protocol_utils::read_until(&mut reader, protocol_utils::REQUEST_LENGTH_CERO, false)?;
        for request_received in requests_received {
            if let [prev_remote_hash, new_remote_hash, branch_name] = request_received.split_whitespace().collect::<Vec<&str>>().as_slice() {
                refs_to_update.push((prev_remote_hash.to_string(), new_remote_hash.to_string(), branch_name.to_string()));
            }
        }
        
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        let mut file = File::create(".git/pack/received_pack_file.pack")?;
        file.write_all(&buffer)?;

        println!("received packfile");

        match UnpackObjects::new().execute(Some(vec![".git/pack/received_pack_file.pack"])) {
            Ok(_) => {
                let unpack_confirmation = protocol_utils::format_line_to_send(protocol_utils::UNPACK_CONFIRMATION.to_string());
                stream.write_all(unpack_confirmation.as_bytes())?;
            }
            Err(_) => {}
        }

        println!("packfile unpacked");

        Ok(())
    }

    pub fn clone_from_remote(&self) -> Result<(), Box<dyn Error>> {
        /*let mut stream = TcpStream::connect("127.0.0.1:9418")?;

        let request = format!("{:04x}git-upload-pack /.git\0host=127.0.0.1\0", "git-upload-pack /.git\0host=127.0.0.1\0".len() + 4);
        stream.write_all(request.as_bytes())?;
        stream.flush()?;

        let reader = std::io::BufReader::new(&stream);
        for line in reader.lines() {
            let line = line?;
            println!("{}", line);
            break;
        }

        let branch_path = helpers::get_current_branch_path()?;
        let last_commit_hash: String = helpers::read_file_content(&branch_path)?;

        let line = format!("want {} multi_ack side-band-64k ofs-delta", last_commit_hash);
        let actual_line = format!("{:04x}{}\n", line.len() + 5, line);
        println!("{}", actual_line);
        stream.write_all(actual_line.as_bytes())?;
        stream.write_all("0000".as_bytes())?;
        let done = format!("{:04x}done\n", "done\n".len()+4);
        println!("{}", done);
        stream.write_all(done.as_bytes())?;
        stream.flush()?;

        Ok(())*/
        Ok(())
    }
}
