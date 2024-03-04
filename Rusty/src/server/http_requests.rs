use crate::server::server_protocol::ServerProtocol;
use crate::commands::git_commands::{Command, Log, Merge, PathHandler};
use crate::commands::helpers;
use crate::constants::{HTTP_RESPONSE_ERROR, HTTP_RESPONSE_SUCCESFUL, ALL_BRANCHES_LOCK, SERVER_BASE_PATH, PULL_REQUEST_FILE, SEPARATOR_PULL_REQUEST_FILE};
use std::{error::Error, fs::File, io, io::Read, io::Write, net::TcpStream, borrow::Cow, fs::OpenOptions};
use serde::{Serialize, Deserialize};
use crate::commands::helpers::{get_branch_last_commit, get_branch_path};
use crate::server::locked_branches_manager::*;
use std::{collections::HashSet, sync::{Mutex, Arc, Condvar}, thread};


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

pub struct HttpRequestHandler;

impl HttpRequestHandler {
    pub fn handle_api_requests(locked_branches: &Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
        let listener = ServerProtocol::bind("127.0.0.1:8081")?; // Default Git port
        println!("bind api complete");
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let cloned_locked_branches = Arc::clone(locked_branches);
                    let mut cloned_stream = stream.try_clone()?;
                    let mut path_handler = PathHandler::new(SERVER_BASE_PATH.to_string());
                    thread::spawn(move || {
                        if let Err(err) = HttpRequestHandler::endpoint_handler(&mut cloned_stream, &mut path_handler, cloned_locked_branches) {
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
                Ok(pull_request)
            }
            Err(e) => {
                Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Error deserializing JSON: {}", e),
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
                }
            }
        }
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Error: JSON object not found in the received data.",
        )))
    }

    pub fn merge_pull_request(request: Cow<str>, pull_request_path: &str, _request_url: &str, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let split_request: Vec<&str> = request.split_whitespace().collect();
        let url = split_request[1];
        println!("my url: {}", url);

        let split_url: Vec<&str> = url.split('/').collect();
        let pull_request_id = split_url[4];

        let _body = HttpRequestHandler::get_body(request.clone())?;

        let file_content = helpers::read_file_content(&path_handler.get_relative_path(pull_request_path))?;

        let mut merge_hash = String::new();
        let mut new_file_content_lines = Vec::new();

        let file_content_lines: Vec<&str> = file_content.split('\n').collect();
        for line in file_content_lines {
            if line.is_empty() {
                continue;
            }
            println!("line in file: {}", line);
            let mut pr: PullRequest = HttpRequestHandler::deserialize_pull_request(line.to_string())?;

            if pr.id == pull_request_id {
                merge_hash = Merge::new().execute(Some(vec![&pr.head, &pr.base]), path_handler)?;
                pr.commit_after_merge = merge_hash.clone();
                new_file_content_lines.push(serde_json::to_string(&pr)?);
            } else {
                new_file_content_lines.push(line.to_string());
            }
        }

        let mut file = File::create(path_handler.get_relative_path(pull_request_path))?;
        file.write_all(new_file_content_lines.join("\n").as_bytes())?;

        Ok(merge_hash)
    }

    pub fn add_pull_request(request: Cow<str>, pull_request_path: &str, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
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

        match file.write_all(format!("{}{}", HttpRequestHandler::get_body(request)?, SEPARATOR_PULL_REQUEST_FILE).as_bytes()) {
            Ok(_) => println!("Content written to file successfully."),
            Err(e) => eprintln!("Error writing to file: {}", e),
        }
        Ok("Pull Request added.".to_string())
    }

    pub fn get_pull_request(pull_request: Option<&str>, pull_request_path: &str, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let file_content = helpers::read_file_content(&path_handler.get_relative_path(pull_request_path))?;
        let mut pull_requests_response = Vec::new();

        let file_content_lines: Vec<&str> = file_content.split('\n').collect();
        for line in file_content_lines {
            if line.is_empty() {
                continue;
            }
            let pr: PullRequest = HttpRequestHandler::deserialize_pull_request(line.to_string())?;
            let pull_request_id: &str = if pull_request.is_some() {
                pull_request.unwrap_or("")
            } else { &pr.id };
            if pr.id == pull_request_id {
                pull_requests_response.push(serde_json::to_string(&pr)?);
            }
        }

        Ok(pull_requests_response.join("\n"))
    }

    pub fn get_pull_request_logs(pull_request: Option<&str>, pull_request_path: &str, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        println!("log_pull_request id {:?}", pull_request);
        let file_content = helpers::read_file_content(&path_handler.get_relative_path(pull_request_path))?;

        let file_content_lines: Vec<&str> = file_content.split('\n').collect();
        for line in file_content_lines {
            if line.is_empty() {
                continue;
            }
            let pr: PullRequest = HttpRequestHandler::deserialize_pull_request(line.to_string())?;
            let pull_request_id = match pull_request {
                Some(id) => id,
                None =>
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: can't log pull request with empty id",
                    )))
            };
            if pr.id == pull_request_id {
                //new_file_content_lines.push(serde_json::to_string(&pr)?);
                if pr.commit_after_merge.is_empty() {
                    println!("commit after merge is empty");
                    let head_last_commit = match get_branch_last_commit(&get_branch_path(&pr.head), path_handler) {
                        Ok(commit) => commit,
                        Err(e) => {
                            return Err(Box::new(io::Error::new(
                                io::ErrorKind::Other,
                                format!("Error finding last commit of branch {}, {}", &pr.head, e),
                            )));
                        }
                    };
                    let base_last_commit = match get_branch_last_commit(&get_branch_path(&pr.base), path_handler) {
                        Ok(commit) => commit,
                        Err(e) => {
                            return Err(Box::new(io::Error::new(
                                io::ErrorKind::Other,
                                format!("Error finding last commit of branch {}, {}", &pr.base, e),
                            )));
                        }
                    };
                    let logs_head = Log::new().execute(Some(vec![&head_last_commit]), path_handler)?;
                    let logs_base = Log::new().execute(Some(vec![&base_last_commit]), path_handler)?;
                    return Ok(format!("\n-- PullRequest Head '{}' Log --\n{}\n\n-- PullRequest Base '{}' Log --\n{}", &pr.head,  logs_head, &pr.base,  logs_base));
                } 
                return Log::new().execute(Some(vec![&pr.commit_after_merge]), path_handler);
            }
        }

        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Error: Error generating logs",
        )))
    }

    pub fn handle_get_request(_request: Cow<str>, pull_request_path: &str, request_url: &str, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        // Split the string by "/"
        let params: Vec<&str> = request_url.split('/').collect();

        match params.len() {
            4..=6 => {
                let pull_request = if params.len() > 4 { Some(params[4]) } else { None };

                if params[3] != "pulls" {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Invalid params in url params 3 is not pulls",
                    )));
                }
                let extra_param = if params.len() == 6 { Some(params[5]) } else { None };
                println!("Pull Request: {:?}, Extra Param: {:?}", pull_request, extra_param);
                if params.len() == 4 {
                    return HttpRequestHandler::get_pull_request(None, pull_request_path, path_handler);
                }

                if pull_request.is_none() {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Invalid params in url pull_request id is none",
                    )))
                }

                if params.len() == 5 {
                    return HttpRequestHandler::get_pull_request(pull_request, pull_request_path, path_handler);
                }

                if params.len() == 6 {
                    if extra_param.is_none() {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::Other,
                            "Error: Invalid params in url extra param is none",
                        )))
                    }
                    if let Some(extra_param) = extra_param {
                        if extra_param != "commits" {
                            return Err(Box::new(io::Error::new(
                                io::ErrorKind::Other,
                                "Error: Invalid params in url extra param is not commits",
                            )))

                        }
                        return HttpRequestHandler::get_pull_request_logs(pull_request, pull_request_path, path_handler);

                    }
                }
                Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Invalid params in url, no one handled request",
                )))



            }
            _ => {
                Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Invalid number of params in url",
                )))
            }
        }
    }

    // TODO change return to http response
    pub fn handle_http(request: Cow<str>, request_type: HttpRequestType, request_url: &str, path_handler: &mut PathHandler) -> Result<String, Box<dyn Error>> {

        let params: Vec<&str> = request_url.split('/').collect();

        if params.len() < 2 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Invalid params in url < 2",
            )));
        }
        if params[1] != "repos" {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Invalid params in url first param not repos",
            )));
        }

        let repo_name = if params.len() > 2 { Some(params[2]) } else { None };
        let repo = match repo_name {
            Some(repo) => repo,
            _ =>
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Couldn't parse repo name from url",
                ))),
        };
        path_handler.set_relative_path(path_handler.get_relative_path(repo));
        match request_type {
            HttpRequestType::GET => {
                HttpRequestHandler::handle_get_request(request, PULL_REQUEST_FILE, request_url, path_handler)

            },
            HttpRequestType::POST => {
                HttpRequestHandler::add_pull_request(request, PULL_REQUEST_FILE, path_handler)
            },
            HttpRequestType::PUT => {
                HttpRequestHandler::merge_pull_request(request, PULL_REQUEST_FILE, request_url, path_handler)
            },
        }
    }

    pub fn endpoint_handler(stream: &mut TcpStream, path_handler: &mut PathHandler, locked_branches: Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
        // read client request

        let mut locked_branches_lifetime = LockedBranches::new(&locked_branches);
        locked_branches_lifetime.lock_branch(ALL_BRANCHES_LOCK, false)?;

        let mut buffer = [0; 1024];
        stream.read_exact(&mut buffer).expect("Failed to read");

        let request = String::from_utf8_lossy(&buffer[..]);
        let request_type: &str = request.split_whitespace().next().unwrap_or("");
        let request_url: &str = request.split_whitespace().nth(1).unwrap_or_default();
        println!("req_type: -{}-", request_type);
        println!("req_url: -{}-", request_url);
        let http_request_type = HttpRequestType::new(request_type);
        let final_response = match HttpRequestHandler::handle_http(request.clone(), http_request_type, request_url, path_handler) {
            Ok(http_response) => format!("{}\r\nContent-Length: {}\r\n\r\n{}", HTTP_RESPONSE_SUCCESFUL, http_response.len(), http_response),
            Err(_) => format!("{}\r\n\r\n\r\n", HTTP_RESPONSE_ERROR),
        };
        println!("sending response to client: {}", final_response);

        let _ = stream.write(final_response.as_bytes());
        let _ = stream.flush();

        drop(locked_branches_lifetime);

        Ok(())
    }
}