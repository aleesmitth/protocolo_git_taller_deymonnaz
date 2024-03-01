use crate::commands::git_commands::Command;
use crate::commands::git_commands::PackObjects;
use crate::commands::git_commands::PathHandler;
use crate::commands::git_commands::UnpackObjects;
use crate::commands::helpers;
use crate::commands::protocol_utils;
use crate::constants::{REQUEST_LENGTH_CERO, REQUEST_DELIMITER_DONE, WANT_REQUEST, NAK_RESPONSE, UNPACK_CONFIRMATION};

use std::{error::Error, fs::File, io, io::Read, io::Write, net::TcpListener, net::TcpStream};
const RECEIVE_PACK: &str = "git-receive-pack";
const UPLOAD_PACK: &str = "git-upload-pack";

pub struct ServerProtocol;

impl Default for ServerProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerProtocol {
    pub fn new() -> Self {
        ServerProtocol {}
    }

    pub fn bind(address: &str) -> Result<TcpListener, Box<dyn std::error::Error>> {
        println!("binding to client...");
        Ok(TcpListener::bind(address)?)
    }

    pub fn handle_client_conection(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let stream_clone = stream.try_clone()?;
        let mut reader = std::io::BufReader::new(stream_clone);
        println!("waiting for request...");
        let request_length = protocol_utils::get_request_length(&mut reader)?;
        if request_length == 0 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Request length 0",
            )));
        }
        println!("reading request_...");
        let request = protocol_utils::read_exact_length_to_string(&mut reader, request_length)?;
        let request_array: Vec<&str> = request.split_whitespace().collect();
        let second_part_of_request = request
            .split('\0')
            .next() // Take the first element before '\0'
            .and_then(|s| s.split_whitespace().nth(1)) // Extract the second part after the space
            .map_or_else(|| "/".to_string(), String::from);

        let trimmed_string_path = if let Some(stripped) = second_part_of_request.strip_prefix('/') {
            stripped
        } else {
            second_part_of_request.as_str()
        };

        let result_string = trimmed_string_path.to_string();

        let trimmed_path_name = result_string.strip_suffix(".git").unwrap_or(result_string.as_str());

        // Retrieve the current value
        let base_repo_path = PathHandler::get_relative_path("");
        let mut current_repo_path = base_repo_path.clone();

        // Concatenate a new string
        current_repo_path.push_str(trimmed_path_name);
        println!("current_repo_path: {:?}", current_repo_path);
        
        // Check if the directory exists
    /*if !fs::metadata(&current_repo_path).is_ok() {
        // If the directory doesn't exist, create it
        if let Err(err) = fs::create_dir(&current_repo_path) {
            eprintln!("Error creating directory: {}", err);
        } else {
            println!("Directory created successfully!");
        }
    } else {
        println!("Directory already exists.");
    }*/
        // Set the modified value back to the environment variable
        // TODO important, you can't do this because env variables are shared among threads
        PathHandler::set_relative_path(&current_repo_path);



        match request_array[0] {
            UPLOAD_PACK => {
                if let Err(err) = ServerProtocol::upload_pack(stream) {
                    eprintln!("Error handling UPLOAD_PACK: {}", err);
                }
            },
            RECEIVE_PACK => {
                if let Err(err) = ServerProtocol::receive_pack(stream) {
                    eprintln!("Error handling RECEIVE_PACK: {}", err);
                }
            },
            _ => {}
        }

        PathHandler::set_relative_path(&base_repo_path);
        println!("end handling connection, relative path reseted.");
        Ok(())
    }

    pub fn upload_pack(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        println!("git-upload-pack");
        let branches: Vec<String> = helpers::get_all_branches()?;
        for branch in &branches {
            let line_to_send = protocol_utils::format_line_to_send(branch.clone());
            // println!("sending line: {:?}", line_to_send);
            stream.write_all(line_to_send.as_bytes())?;
        }

        let _ = stream.write_all(REQUEST_LENGTH_CERO.as_bytes());

        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        let requests_received: Vec<String> =
            protocol_utils::read_until(&mut reader, REQUEST_DELIMITER_DONE, false)?;
        for request_received in requests_received.clone() {
            let request_array: Vec<&str> = request_received.split_whitespace().collect();
            // println!("request in array: {:?}", request_array);
            if request_array[0] != WANT_REQUEST {
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

        let _ = stream.write_all(
            protocol_utils::format_line_to_send(NAK_RESPONSE.to_string())
                .as_bytes(),
        );
        println!("sent NAK");
        // let _: Vec<String> =
        //     protocol_utils::read_until(&mut reader, protocol_utils::REQUEST_DELIMITER_DONE, false)?;
        println!("received done");
        let mut commits: Vec<String> = Vec::new();

        for request_received in requests_received {
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

        pack_file.read_to_end(&mut buffer)?;

        stream.write_all(&buffer)?;
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

        let _ = stream.write_all(REQUEST_LENGTH_CERO.as_bytes());

        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        let requests_received: Vec<String> =
            protocol_utils::read_until(&mut reader, REQUEST_LENGTH_CERO, true)?;
        let mut refs_to_update: Vec<(String, String, String)> = Vec::new();
        for request_received in requests_received {
            if let [prev_remote_hash, new_remote_hash, branch_name] = request_received
                .split_whitespace()
                .collect::<Vec<&str>>()
                .as_slice()
            {
                helpers::validate_ref_update_request(
                    prev_remote_hash,
                    new_remote_hash,
                    branch_name,
                )?;
                refs_to_update.push((
                    prev_remote_hash.to_string(),
                    new_remote_hash.to_string(),
                    branch_name.to_string(),
                ));
            }
        }

        println!("reading pack file");

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        let mut file = File::create(PathHandler::get_relative_path(
            ".git/pack/received_pack_file.pack",
        ))?;

        file.write_all(&buffer)?;

        if UnpackObjects::new()
            .execute(Some(vec![&PathHandler::get_relative_path(
                ".git/pack/received_pack_file.pack",
            )]))
            .is_ok()
        {
            println!("packfile unpacked");
            let unpack_confirmation = protocol_utils::format_line_to_send(
                UNPACK_CONFIRMATION.to_string(),
            );
            println!("{}", unpack_confirmation);
            stream.write_all(unpack_confirmation.as_bytes())?;
        }
        helpers::update_hash_for_refs(refs_to_update)?;

        Ok(())
    }
}
