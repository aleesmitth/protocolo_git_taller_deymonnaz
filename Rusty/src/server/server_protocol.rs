use crate::commands::git_commands::{Command, Merge, PackObjects, PathHandler, UnpackObjects};
use crate::commands::helpers;
use crate::commands::protocol_utils;
use crate::server::locked_branches_manager::{self, *};
use std::{collections::HashSet, sync::{Mutex, Arc, Condvar}, thread};
use crate::constants::{REQUEST_LENGTH_CERO, REQUEST_DELIMITER_DONE, WANT_REQUEST, NAK_RESPONSE, UNPACK_CONFIRMATION, ALL_BRANCHES_LOCK, SERVER_BASE_PATH, PULL_REQUEST_FILE, SEPARATOR_PULL_REQUEST_FILE};
use std::{error::Error, fs::File, io, io::Read, io::Write, net::TcpListener, net::TcpStream};
use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::BufRead;

const RECEIVE_PACK: &str = "git-receive-pack";
const UPLOAD_PACK: &str = "git-upload-pack";

pub struct ServerProtocol;

impl Default for ServerProtocol {
    fn default() -> Self {
        Self::new()
    }
}

use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize,Deserialize)]
pub struct PullRequest {
    id: String,
    title: String,
    body: String,
    head: String,
    base: String,
    repo: String,
    commit_after_merge: String,

}

pub enum HttpRequestType {
    POST,
    PUT,
    GET,
}
impl HttpRequestType {
    fn new(method: &str) -> Self {
        match method {
            "POST" => HttpRequestType::POST,
            "PUT" => HttpRequestType::PUT,
            "GET" => HttpRequestType::GET,
            _ => {
                eprintln!("Unexpected string: {}", method);
                HttpRequestType::GET
            }
        }
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

    pub fn handle_api_requests(locked_branches: &Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
        let listener = ServerProtocol::bind("127.0.0.1:8081")?; // Default Git port
        println!("bind api complete");

        // Create a HashSet to store locked branch names
        //let locked_branches = Arc::new((Mutex::new(HashSet::new()), Condvar::new()));
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let cloned_locked_branches = Arc::clone(&locked_branches);
                    let mut cloned_stream = stream.try_clone()?;
                    let mut path_handler = PathHandler::new(SERVER_BASE_PATH.to_string());
                    thread::spawn(move || {
                        if let Err(err) = ServerProtocol::endpoint_handler(&mut cloned_stream, &mut path_handler, cloned_locked_branches) {
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

    pub fn deserialize_pull_request(json: String) -> Result<PullRequest, Box<dyn Error>> {
        // Deserialize the JSON data into a PullRequest object
        match serde_json::from_str::<PullRequest>(&json) {
            Ok(pull_request) => {
                println!("Parsed PullRequest: {:?}", pull_request);
                return Ok(pull_request)
            }
            Err(e) => {
                eprintln!("Error deserializing JSON: {}", e);
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: @@@@@@@@@@@@",
                )))
            }
        }
    }

    pub fn get_body(request: Cow<str>) -> Result<String, Box<dyn Error>> {
        if let Some(body_index) = request.find("\r\n\r\n") {
            let body = &request[body_index + 4..];

            if let Some(json_start) = body.find('{') {
                let json_body = &body[json_start..];

                // Trim the JSON string to remove leading and trailing whitespaces, newlines, etc.
                let trimmed_json = json_body.trim();

                println!("Received JSON body: .{}.", trimmed_json);
                let start_index = trimmed_json.find('{');
                let end_index = trimmed_json.rfind('}');

                if let (Some(start), Some(end)) = (start_index, end_index) {
                    // Extract the JSON substring
                    let json_str = &trimmed_json[start..=end];
                    return Ok(json_str.to_string());
                } else {
                    eprintln!("JSON object not found in the received data.");
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: @@@@@@@@@@@@",
                    )))
                }
            }
        }
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: @@@@@@@@@@@@",
            )))
    }

    pub fn merge_pull_request(request: Cow<str>, pull_request_path: &str, request_url: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        let split_request: Vec<&str> = request.split_whitespace().collect();
        let url = split_request[1];
        println!("my url: {}", url);

        let split_url: Vec<&str> = url.split('/').collect();
        let pull_request_id = split_url[4];

        let body = ServerProtocol::get_body(request.clone())?;
        println!("body: {}", body);

        let file_content = helpers::read_file_content(&path_handler.get_relative_path(pull_request_path))?;
        // tenes repo tenes id de PR
        let mut merge_hash = String::new();
        let mut new_file_content_lines = Vec::new();

        let file_content_lines: Vec<&str> = file_content.split('\n').collect();
        for line in file_content_lines {
            if line.is_empty() {
                continue;
            }
            println!("line in file: {}", line);
            let mut pr: PullRequest = ServerProtocol::deserialize_pull_request(line.to_string())?;

            if pr.id == pull_request_id {
                println!("Aca");
                // merge_hash = Merge::new().execute(Some(vec![&pr.head, &pr.base]), path_handler)?;
                merge_hash = "martin".to_string();
                println!("merge {}", merge_hash);
                pr.commit_after_merge = merge_hash;
                new_file_content_lines.push(serde_json::to_string(&pr)?);
            } else {
                new_file_content_lines.push(line.to_string());
            }
        }

        let mut file = File::create(&path_handler.get_relative_path(pull_request_path))?;
        file.write_all(new_file_content_lines.join("\n").as_bytes())?;

        Ok(())
    }

    pub fn add_pull_request(request: Cow<str>, pull_request_path: &str, request_url: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        println!("testestest");
        let mut file = match OpenOptions::new()
            .append(true)
            .create(true)
            .write(true)
            .open(path_handler.get_relative_path(pull_request_path)) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error creating file: {}", e);
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Can't open pull request file",
                )));
            }
        };
        println!("testestest");

        match file.write_all(format!("{}{}", ServerProtocol::get_body(request)?, SEPARATOR_PULL_REQUEST_FILE).as_bytes()) {
            Ok(_) => println!("Content written to file successfully."),
            Err(e) => eprintln!("Error writing to file: {}", e),
        }
        Ok(())
    }

    pub fn get_pull_request(repo_name: &str, pull_request: Option<&str>, pull_request_path: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        // TODO read file of PR, search by repo + id, but if id is empty search by repo only
        println!("get_pull_request repo {} id {:?}", repo_name, pull_request);
        Ok(())

    }

    pub fn get_pull_request_logs(repo_name: &str, pull_request: Option<&str>, pull_request_path: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        // TODO find pull request by repo + id, if it has merged log the merge commit,
        // TODO otherwise log the head and the base separately
        println!("log_pull_request repo {} id {:?}", repo_name, pull_request);
        Ok(())

    }

    pub fn handle_get_request(request: Cow<str>, pull_request_path: &str, request_url: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        // Split the string by "/"
        let params: Vec<&str> = request_url.split('/').collect();

        match params.len() {
            4..=5 => {
                if params.len() < 2 {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Invalid params in url",
                    )));
                }
                if params[1] != "repos" {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Invalid params in url",
                    )));
                }
                let repo_name = if params.len() > 2 { Some(params[2]) } else { None };
                let pull_request = if params.len() > 3 { Some(params[4]) } else { None };

                if repo_name.is_none() {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Invalid params in url",
                    )));
                }

                let repo_name = repo_name.unwrap_or("");

                if params[3] != "pulls" {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Invalid params in url",
                    )));
                }
                let extra_param = if params.len() == 5 { Some(params[5]) } else { None };
                println!("Repo Name: {}, Pull Request: {:?}, Extra Param: {:?}", repo_name, pull_request, extra_param);

                if params.len() == 3 {
                    return ServerProtocol::get_pull_request(repo_name, None, pull_request_path, path_handler);
                }

                if pull_request.is_none() {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Invalid params in url",
                    )));
                }

                if params.len() == 4 {
                    return ServerProtocol::get_pull_request(repo_name, pull_request, pull_request_path, path_handler);
                }

                if params.len() == 5 {
                    if extra_param.is_none() {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::Other,
                            "Error: Invalid params in url",
                        )));
                    }
                    if let Some(extra_param) = extra_param {
                        if extra_param != "commit" {
                            return Err(Box::new(io::Error::new(
                                io::ErrorKind::Other,
                                "Error: Invalid params in url",
                            )));

                        }
                        return ServerProtocol::get_pull_request_logs(repo_name, pull_request, pull_request_path, path_handler);

                    }
                }
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Invalid params in url",
                )));



            }
            _ => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Invalid number of params in url",
                )));
            }
        }
        Ok(())
    }

    // TODO change return to http response
    pub fn handle_http(request: Cow<str>, request_type: HttpRequestType, request_url: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        match request_type {
            HttpRequestType::GET => {
                // TODO make it so that it returns the http response
                ServerProtocol::handle_get_request(request, request_url, PULL_REQUEST_FILE, path_handler)?

            },
            HttpRequestType::POST => {
                // TODO leer parametros para saber en q repo va
                ServerProtocol::add_pull_request(request, request_url, PULL_REQUEST_FILE, path_handler)?
            },
            HttpRequestType::PUT => {
                // TODO usar el repo name tambien
                ServerProtocol::merge_pull_request(request, request_url, PULL_REQUEST_FILE, path_handler)?
            },
        }
        Ok(())
    }

    pub fn endpoint_handler(stream: &mut TcpStream, path_handler: &mut PathHandler, locked_branches: Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
        // read client request

        let mut locked_branches_lifetime = LockedBranches::new(&locked_branches);
        locked_branches_lifetime.lock_branch(ALL_BRANCHES_LOCK, false)?;

        let stream_clone = stream.try_clone()?;
        //let mut reader = std::io::BufReader::new(stream_clone);
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).expect("Failed to read");
        println!("buffer header:{:?}", buffer);

        let request = String::from_utf8_lossy(&buffer[..]);
        let request_type: &str = request.split_whitespace().next().unwrap_or("");
        let request_url: &str = request.split_whitespace().nth(1).unwrap_or_default();
        println!("req_type: -{}-", request_type);
        println!("req_url: -{}-", request_url);
        let http_request_type = HttpRequestType::new(request_type);
        println!("req: @{}@", request.clone());
        ServerProtocol::handle_http(request.clone(), http_request_type, request_url, path_handler)?;

        //println!("req: @{}@", request);

        //ServerProtocol::get_body(request);

        drop(locked_branches_lifetime);

        Ok(())
    }

    pub fn handle_client_connection(stream: &mut TcpStream, path_handler: &mut PathHandler, locked_branches: Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
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

        path_handler.set_relative_path(path_handler.get_relative_path(trimmed_path_name));

        match request_array[0] {
            UPLOAD_PACK => {
                if let Err(err) = ServerProtocol::upload_pack(stream, path_handler, &locked_branches) {
                    eprintln!("Error handling UPLOAD_PACK: {}", err);
                }
            },
            RECEIVE_PACK => {
                if let Err(err) = ServerProtocol::receive_pack(stream, path_handler, &locked_branches) {
                    eprintln!("Error handling RECEIVE_PACK: {}", err);
                }
            },
            _ => {}
        }

        println!("end handling connection, relative path reseted.");
        Ok(())
    }

    

    pub fn upload_pack(stream: &mut TcpStream, path_handler: &PathHandler, locked_branches: &Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
        println!("git-upload-pack");

        let mut locked_branches_lifetime = LockedBranches::new(locked_branches);

        locked_branches_lifetime.lock_branch(ALL_BRANCHES_LOCK, false)?;
        
        let branches: Vec<String> = helpers::get_all_branches(path_handler)?;
        for branch in &branches {
            let line_to_send = protocol_utils::format_line_to_send(branch.clone());
            // println!("sending line: {:?}", line_to_send);
            stream.write_all(line_to_send.as_bytes())?;
        }

        let _ = stream.write_all(REQUEST_LENGTH_CERO.as_bytes());

        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        let requests_received: Vec<String> =
            protocol_utils::read_until(&mut reader, REQUEST_DELIMITER_DONE, false)?;
        let mut branches_used: HashSet<String> = HashSet::new();
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

            let valid_branches = match ServerProtocol::validate_is_latest_commit_any_branch(request_array[1], &branches) {
                Ok(branches_used) => branches_used,
                Err(e) => {
                    println!("invalid commit: {:?}", request_array);
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Error: Received invalid commit hash for want request {}", e),
                    )));
                }
            };

            branches_used.extend(valid_branches);
            println!("valid want request.");
        }

        for branch in &branches_used {
            locked_branches_lifetime.lock_branch(branch, true)?;
        }

        locked_branches_lifetime.unlock_branch(ALL_BRANCHES_LOCK)?;

        let _ = stream.write_all(
            protocol_utils::format_line_to_send(NAK_RESPONSE.to_string())
                .as_bytes(),
        );
        println!("sent NAK");

        let mut commits: Vec<String> = Vec::new();
        for request_received in requests_received {
            let request_array: Vec<&str> = request_received.split_whitespace().collect();
            if let Some(second_element) = request_array.get(1) {
                commits.push(second_element.to_string());
            }
        }

        let commits_str: Vec<&str> = commits.iter().map(|s| s.as_str()).collect();
        let checksum = PackObjects::new().execute(Some(commits_str.clone()), path_handler)?;
        let pack_file_path = format!(".git/pack/pack-{}.pack", checksum);
        let mut pack_file = File::open(path_handler.get_relative_path(&pack_file_path))?;
        let mut buffer = Vec::new();

        pack_file.read_to_end(&mut buffer)?;

        stream.write_all(&buffer)?;

        drop(locked_branches_lifetime);

        println!("sent pack file");

        Ok(())
    }

    pub fn validate_is_latest_commit_any_branch(commit: &str, branches: &Vec<String>) -> Result<Vec<String>, Box<dyn Error>> {
        let mut valid_branches: Vec<String> = Vec::new();
        for branch in branches {
            // Split the string into words
            let branch_commit_and_name: Vec<&str> = branch.split_whitespace().collect();
            if let Some(first_word) = branch_commit_and_name.first() {
                if first_word == &commit {
                    valid_branches.push(branch.to_string());
                }
            }
        }

        if !valid_branches.is_empty() {
            return Ok(valid_branches)
        }

        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Error: Commit {} is not the latest commit in any branch", commit),
        )))
    }

    pub fn receive_pack(stream: &mut TcpStream, path_handler: &PathHandler, locked_branches: &Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
        println!("git-receive-pack");

        let mut locked_branches_lifetime = LockedBranches::new(locked_branches);

        locked_branches_lifetime.lock_branch(ALL_BRANCHES_LOCK, false)?;      

        let branches: Vec<String> = match helpers::get_all_branches(path_handler) {
            Ok(branches) => branches,
            Err(e) => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    e.to_string(),
                )))
            }
        };
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
        // let mut branches_used: HashSet<String> = HashSet::new();
        for request_received in requests_received {
            if let [prev_remote_hash, new_remote_hash, branch_name] = request_received
                .split_whitespace()
                .collect::<Vec<&str>>()
                .as_slice()
            {

                locked_branches_lifetime.lock_branch(branch_name, true)?;

                helpers::validate_ref_update_request(
                    prev_remote_hash,
                    new_remote_hash,
                    branch_name,
                    path_handler
                )?;
                refs_to_update.push((
                    prev_remote_hash.to_string(),
                    new_remote_hash.to_string(),
                    branch_name.to_string(),
                ));
            }
        }

        locked_branches_lifetime.unlock_branch(ALL_BRANCHES_LOCK)?;

        println!("reading pack file");

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        let mut file = File::create(path_handler.get_relative_path(
            ".git/pack/received_pack_file.pack",
        ))?;

        file.write_all(&buffer)?;

        if UnpackObjects::new()
            .execute(Some(vec![&path_handler.get_relative_path(
                ".git/pack/received_pack_file.pack",
            )]), path_handler)
            .is_ok()
        {
            println!("packfile unpacked");
            let unpack_confirmation = protocol_utils::format_line_to_send(
                UNPACK_CONFIRMATION.to_string(),
            );
            println!("unpack confirmation: {}", unpack_confirmation);
            stream.write_all(unpack_confirmation.as_bytes())?;
        }
        helpers::update_hash_for_refs(refs_to_update, path_handler)?;

        drop(locked_branches_lifetime);

        Ok(())
    }
}