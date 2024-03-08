use crate::server::server_protocol::ServerProtocol;
use crate::commands::git_commands::{Command, Log, Merge, PathHandler};
use crate::commands::helpers;
use crate::constants::{ALL_BRANCHES_LOCK, API_PORT, CONTENT_TYPE, DEFAULT_BRANCH_NAME, HTTP_VERSION, IP_LOCALHOST, PR_MERGE_SUCCESS, PULL_REQUEST_FILE, SEPARATOR_PULL_REQUEST_FILE, SERVER_BASE_PATH};
use std::fmt;
use chrono::{Utc, DateTime};
use std::{error::Error, fs::File, io::Read, io::Write, net::TcpStream, borrow::Cow, fs::OpenOptions};
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

#[derive(Debug, Serialize,Deserialize)]
pub struct RepoResponse {
    name: String,
    default_branch: String
}

#[derive(Debug, Serialize,Deserialize)]
pub struct BranchResponse {
    label: String,
    sha: String,
    repo: RepoResponse
}

#[derive(Debug, Serialize,Deserialize)]
pub struct MergeResponseType {
    sha: String,
    merged: bool,
    message: String
}

#[derive(Debug, Serialize,Deserialize)]
pub struct LogResponseType {
    sha: String,
    commit: CommitResponse,
    parents: Vec<ParentResponse>,
}

#[derive(Debug, Serialize,Deserialize)]
pub struct ParentResponse {
    sha: String,
}

#[derive(Debug, Serialize,Deserialize, Default)]
pub struct UserResponse {
    name: String,
    email: String,
    date: String,
}

#[derive(Debug, Serialize,Deserialize)]
pub struct CommitResponse {
    author: UserResponse,
    committer: UserResponse,
    message: String,
    tree: String,
}

#[derive(Debug, Serialize,Deserialize)]
pub struct ResponseType {
    url: String,
    id: String, // Pull Request Ids
    title: String,
    merge_commit_sha: Option<String>,
    head: BranchResponse,
    base: BranchResponse,
    body: String
}

#[derive(Debug)]
pub enum ResponseStatusCode {
    Forbidden, // 403
    ValidationFailed, // 422
    NotFound, // 404
    InternalError, // 500
    MethodNotAllowed, // 405
    ConflictingSha, // 409
    BadRequest // 400
}

#[derive(Debug)]
pub enum SuccessResponseStatusCode {
    Success, //200
    Created //201
}

impl fmt::Display for SuccessResponseStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            SuccessResponseStatusCode::Success => "200 OK",
            SuccessResponseStatusCode::Created => "201 Created",
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug)]
pub struct SuccessResponse {
    code: SuccessResponseStatusCode,
    body: String,
}

impl RepoResponse {
    pub fn new(name: String) -> Self {
        RepoResponse{
            name,
            default_branch:DEFAULT_BRANCH_NAME.to_string()
        }
    }
}

impl BranchResponse {
    pub fn new(label: String, sha: String, repo: RepoResponse) -> Self {
        BranchResponse{
            label,
            sha,
            repo
        }
    }
}

impl MergeResponseType {
    pub fn new(sha: String, message: String) -> Self {
        MergeResponseType{
            sha,
            merged:true,
            message
        }
    }
}

impl LogResponseType {
    pub fn new(sha: String, commit: CommitResponse, parents: Vec<ParentResponse>) -> Self {
        LogResponseType {
            sha,
            commit,
            parents,
        }
    }
}

impl ParentResponse {
    pub fn new(sha: String) -> Self {
        ParentResponse { sha }
    }
}

impl CommitResponse {
    pub fn new(author: UserResponse, committer: UserResponse, message: String, tree: String) -> Self {
        CommitResponse {
            author,
            committer,
            message,
            tree,
        }
    }
}

impl UserResponse {
    pub fn new(name: String, email: String, date: String) -> Self {
        UserResponse {
            name,
            email,
            date,
        }
    }
}

// impl Default for UserResponse {
//     fn default() -> Self {
//         Self {
//             name: String::new(),
//             email: String::new(),
//             date: String::new(),
//         }
//     }
// }

impl ResponseType {
    pub fn new(url: String, pr: PullRequest, repo_name: String, path_handler: &PathHandler) -> Result<Self, ResponseStatusCode> {
        
        let repo_head = RepoResponse::new(repo_name.clone());
        let repo_base = RepoResponse::new(repo_name);

        let head_sha = match get_branch_last_commit(&get_branch_path(&pr.head), path_handler) {
            Ok(commit) => commit,
            Err(_) => {
                return Err(ResponseStatusCode::InternalError);
            }
        };
        let base_sha = match get_branch_last_commit(&get_branch_path(&pr.base), path_handler) {
            Ok(commit) => commit,
            Err(_) => {
                return Err(ResponseStatusCode::InternalError);
            }
        };

        let head = BranchResponse::new(pr.head, head_sha, repo_head);
        let base = BranchResponse::new(pr.base, base_sha, repo_base);
        let merge_commit = if pr.commit_after_merge.is_empty() {
            None
        } else {
            Some(pr.commit_after_merge)
        };

        Ok(ResponseType{
            url,
            id: pr.id, // Pull Request Id
            title: pr.title,
            merge_commit_sha: merge_commit,
            head,
            base,
            body: pr.body,
        })
    }
}

impl SuccessResponse {
    pub fn new<T>(value: &T, code: SuccessResponseStatusCode) -> Result<Self, ResponseStatusCode>
where
    T: ?Sized + Serialize,
{
    let body = match serde_json::to_string(value){
        Ok(body) => body,
        Err(_) => return Err(ResponseStatusCode::InternalError)
    };
    Ok(SuccessResponse{
        code,
        body
    })
}
}

impl fmt::Display for ResponseStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            ResponseStatusCode::BadRequest => "400 Bad Request",
            ResponseStatusCode::Forbidden => "403 Forbidden", 
            ResponseStatusCode::NotFound => "404 Not Found",
            ResponseStatusCode::MethodNotAllowed => "405 Method Not Allowed",
            ResponseStatusCode::ConflictingSha => "409 Conflict",
            ResponseStatusCode::ValidationFailed => "422 Unprocessable Content",
            ResponseStatusCode::InternalError => "500 Internal Server Error"
            
        };
        write!(f, "{}", string)
    }
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
/* 
impl ResponseType {
    fn new(pr: PullRequest) -> Self {
        ResponseType{

        }
    }
}
*/

pub struct HttpRequestHandler;

impl HttpRequestHandler {
    pub fn handle_api_requests(locked_branches: &Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
        let listener = ServerProtocol::bind(&format!("{}:{}", IP_LOCALHOST, API_PORT))?; // Default Git port
        println!("Bind API complete");
        
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

    pub fn deserialize_pull_request(json: String) -> Result<PullRequest, ResponseStatusCode> {
        // Deserialize the JSON data into a PullRequest object
        match serde_json::from_str::<PullRequest>(&json) {
            Ok(pull_request) => {
                println!("Parsed PullRequest: {:?}", pull_request);
                Ok(pull_request)
            }
            Err(_) => {
                Err(ResponseStatusCode::InternalError)
            }
        }
    }

    pub fn get_body(request: Cow<str>) -> Result<String, ResponseStatusCode> {
        println!("Getting body from req: {:?}", request);
        if let Some(body_index) = request.find("\r\n\r\n") {
            let body = &request[body_index + 4..];

            if let Some(json_start) = body.find('{') {
                let json_body = &body[json_start..];

                // Trim the JSON string to remove leading and trailing whitespaces, newlines, etc.
                let trimmed_json = json_body.trim();

                let start_index = trimmed_json.find('{');
                let end_index = trimmed_json.rfind('}');

                if let (Some(start), Some(end)) = (start_index, end_index) {
                    // Extract the JSON substring
                    let json_str = &trimmed_json[start..=end];
                    return Ok(json_str.to_string());
                }
            }
        }
        println!("Throwing error");
        Err(ResponseStatusCode::BadRequest)
    }

    pub fn merge_pull_request(request: Cow<str>, pull_request_path: &str, _request_url: &str, path_handler: &PathHandler) -> Result<SuccessResponse, ResponseStatusCode> {
        let split_request: Vec<&str> = request.split_whitespace().collect();
        let url = split_request[1];

        let split_url: Vec<&str> = url.split('/').collect();

        match split_url.get(5) {
            Some(merge) => {
                if *merge != "merge" {
                    return Err(ResponseStatusCode::BadRequest);
                }
            }
            None => return Err(ResponseStatusCode::BadRequest)
        }

        let pull_request_id = match split_url.get(4) {
            Some(pr_id) => pr_id,
            None => return Err(ResponseStatusCode::BadRequest),
        };       

        let file_content = match  helpers::read_file_content(&path_handler.get_relative_path(pull_request_path)){
            Ok(file_content) => file_content,
            Err(_) => return Err(ResponseStatusCode::InternalError)
        }; 
        
        let mut merge_hash = String::new();
        let mut new_file_content_lines = Vec::new();

        let file_content_lines: Vec<&str> = file_content.split('\n').collect();
        for line in file_content_lines {
            if line.is_empty() {
                continue;
            }
            let mut pr: PullRequest = HttpRequestHandler::deserialize_pull_request(line.to_string())?;

            if !pr.commit_after_merge.is_empty() {
                eprintln!("Can not merge a pull request again.");
                return Err(ResponseStatusCode::MethodNotAllowed)
            }

            if pr.id == *pull_request_id {
                merge_hash = match Merge::new().execute(Some(vec![&pr.head, &pr.base]), path_handler){
                    Ok(merge_hash) => merge_hash,
                    Err(_) => return Err(ResponseStatusCode::InternalError)
                };
                pr.commit_after_merge = merge_hash.clone();
                if let Ok(seralized_pr) = serde_json::to_string(&pr){
                    new_file_content_lines.push(seralized_pr)
                }

            } else {
                new_file_content_lines.push(line.to_string());
            }
        }

        let mut file = match File::create(path_handler.get_relative_path(pull_request_path)){
            Ok(file) => file,
            Err(_) => return Err(ResponseStatusCode::InternalError)
        };
        if file.write_all(new_file_content_lines.join("\n").as_bytes()).is_err() {
            return Err(ResponseStatusCode::InternalError)
        }

        let merge_response = MergeResponseType::new(merge_hash, PR_MERGE_SUCCESS.to_string());

        SuccessResponse::new(&merge_response, SuccessResponseStatusCode::Success)
    }

    fn pull_request_exists(pull_request_id: &str, pull_request_path: &str, _request_url: &str, path_handler: &PathHandler) -> Result<bool, ResponseStatusCode> {
        let file_content = match helpers::read_file_content(&path_handler.get_relative_path(pull_request_path)){
            Ok(file_content) => file_content,
            Err(_) => return Err(ResponseStatusCode::InternalError)
        };

        let file_content_lines: Vec<&str> = file_content.split('\n').collect();
        for line in file_content_lines {
            if line.is_empty() {
                continue;
            }
            let pr: PullRequest = HttpRequestHandler::deserialize_pull_request(line.to_string())?;
            if pr.id == pull_request_id {
                return Ok(true)
            }
        }
        Ok(false)
    }

    pub fn add_pull_request(request: Cow<str>, pull_request_path: &str, request_url: &str, path_handler: &PathHandler, repo: String) -> Result<SuccessResponse, ResponseStatusCode> {
        let mut file = match OpenOptions::new()
            .append(true)
            .create(true)
            .write(true)
            .open(path_handler.get_relative_path(pull_request_path)) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error creating file: {}", e);
                return Err(ResponseStatusCode::InternalError);
            }
        };
        let body = HttpRequestHandler::get_body(request)?;
        let pr: PullRequest = HttpRequestHandler::deserialize_pull_request(body.clone())?;
        
        if let Ok(true) = HttpRequestHandler::pull_request_exists(&pr.id, pull_request_path, request_url, path_handler) {
            return Err(ResponseStatusCode::InternalError);
        };
        match file.write_all(format!("{}{}", body, SEPARATOR_PULL_REQUEST_FILE).as_bytes()) {
            Ok(_) => println!("Content written to file successfully."),
            Err(e) => eprintln!("Error writing to file: {}", e),
        }

        let response = ResponseType::new(request_url.to_string(), pr, repo, path_handler)?;
        SuccessResponse::new(&response, SuccessResponseStatusCode::Created)
        
    }

    pub fn get_pull_request(pull_request: Option<&str>, request_url: &str, pull_request_path: &str, path_handler: &PathHandler, repo_name: String) -> Result<SuccessResponse, ResponseStatusCode> {
        let file_content = match helpers::read_file_content(&path_handler.get_relative_path(pull_request_path)){
            Ok(file_content) => file_content,
            Err(_) => return  Err(ResponseStatusCode::InternalError)
        };
        let mut pull_requests_response: Vec<ResponseType> = Vec::new();

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

                let response = ResponseType::new(request_url.to_string(), pr, repo_name.clone(), path_handler)?;
                pull_requests_response.push(response)
            }
        }

        if pull_request.is_none() {
            return SuccessResponse::new(&pull_requests_response, SuccessResponseStatusCode::Success)
        }
        SuccessResponse::new(&pull_requests_response[0], SuccessResponseStatusCode::Success)
    }

    pub fn get_pull_request_logs(pull_request: Option<&str>, pull_request_path: &str, path_handler: &PathHandler) -> Result<SuccessResponse, ResponseStatusCode> {
        println!("log_pull_request id {:?}", pull_request);
        let file_content = match helpers::read_file_content(&path_handler.get_relative_path(pull_request_path)){
            Ok(file_content) => file_content,
            Err(_) => return Err(ResponseStatusCode::InternalError)
        };

        let file_content_lines: Vec<&str> = file_content.split('\n').collect();
        for line in file_content_lines {
            if line.is_empty() {
                continue;
            }
            let pr: PullRequest = HttpRequestHandler::deserialize_pull_request(line.to_string())?;
            let pull_request_id = match pull_request {
                Some(id) => id,
                None =>
                    return Err(ResponseStatusCode::ValidationFailed)
            };
            if pr.id == pull_request_id {
                
                if pr.commit_after_merge.is_empty() {
                    println!("commit after merge is empty");
                    
                    let head_last_commit = match get_branch_last_commit(&get_branch_path(&pr.head), path_handler) {
                        Ok(commit) => commit,
                        Err(_) => {
                            return Err(ResponseStatusCode::InternalError);
                        }
                    };
                    let base_last_commit = match get_branch_last_commit(&get_branch_path(&pr.base), path_handler) {
                        Ok(commit) => commit,
                        Err(_) => {
                            return Err(ResponseStatusCode::InternalError);
                        }
                    };

                    let mut logs_head = Vec::new();
                    if Log::generate_log_entries(&mut logs_head, head_last_commit, path_handler).is_err() {
                        return Err(ResponseStatusCode::InternalError)
                    }

                    let mut logs_base = Vec::new();
                    if Log::generate_log_entries(&mut logs_base, base_last_commit, path_handler).is_err() {
                        return Err(ResponseStatusCode::InternalError)
                    }
                    logs_head.extend(logs_base);

                    let log_response = HttpRequestHandler::parse_log(logs_head, path_handler)?;
                    return SuccessResponse::new(&log_response, SuccessResponseStatusCode::Success)
                } 
                let mut log = Vec::new();
                if Log::generate_log_entries(&mut log, pr.commit_after_merge, path_handler).is_err() {
                    return Err(ResponseStatusCode::InternalError)
                }
                let log_response = HttpRequestHandler::parse_log(log, path_handler)?;
                return SuccessResponse::new(&log_response, SuccessResponseStatusCode::Success) 
            }
        }

        Err(ResponseStatusCode::NotFound)
    }

    fn parse_log(log: Vec<(String, String)>, path_handler: &PathHandler) -> Result<Vec<LogResponseType>, ResponseStatusCode> {
        
        let mut log_responses = Vec::new();
        for (commit_hash, log_body) in log {
            let log_lines: Vec<&str> = log_body.split('\n').collect();
            let mut parents = Vec::new();
            let mut author = UserResponse::default();
            let mut committer = UserResponse::default();

            for line in log_lines.clone() {
                let split_line: Vec<&str> = line.split_whitespace().collect();
                if line.is_empty() {
                    continue
                }

                match split_line[0] {
                    "parent" => parents.push(ParentResponse::new(split_line[1].to_string())),
                    "author" => {
                        let author_name = split_line[1];
                        let author_email = split_line[2];
                        let author_date = split_line[3];
                        author = UserResponse::new(author_name.to_string(), author_email.to_string(), author_date.to_string())
                    }// aca tengo el username en split_line[1], el mail en split_line[2] y el timestamp en split_line[3]
                    "committer" => {
                        let username = split_line[1];
                        let email = split_line[2];
                        let date = split_line[3];
                        committer = UserResponse::new(username.to_string(), email.to_string(), date.to_string())
                    }
                    _ => {}
                }
            }
            let tree_sha = match helpers::get_commit_tree(&commit_hash, path_handler) {
                Ok(tree_sha) => tree_sha,
                Err(_) => return Err(ResponseStatusCode::InternalError)
            };

            let message = match log_lines.last() {
                Some(message) => message.to_string(),
                None => String::new(),
            };

            let commit_response = CommitResponse::new(author, committer, message, tree_sha);
            log_responses.push(LogResponseType::new(commit_hash, commit_response, parents));
        }
    Ok(log_responses)
    }

    pub fn handle_get_request(_request: Cow<str>, pull_request_path: &str, request_url: &str, path_handler: &PathHandler, repo_name: String) -> Result<SuccessResponse, ResponseStatusCode> {
        // Split the string by "/"
        let params: Vec<&str> = request_url.split('/').collect();

        match params.len() {
            4..=6 => {
                let pull_request = if params.len() > 4 { Some(params[4]) } else { None };

                if params[3] != "pulls" {
                    return Err(ResponseStatusCode::BadRequest);
                }
                let extra_param = if params.len() == 6 { Some(params[5]) } else { None };
                println!("Pull Request: {:?}, Extra Param: {:?}", pull_request, extra_param);
                if params.len() == 4 {
                    return HttpRequestHandler::get_pull_request(None, request_url, pull_request_path, path_handler, repo_name);
                }

                if pull_request.is_none() {
                    return Err(ResponseStatusCode::BadRequest)
                }

                if params.len() == 5 {
                    return HttpRequestHandler::get_pull_request(pull_request, request_url, pull_request_path, path_handler, repo_name);
                }

                if params.len() == 6 {
                    if extra_param.is_none() {
                        return Err(ResponseStatusCode::BadRequest)
                    }
                    if let Some(extra_param) = extra_param {
                        if extra_param != "commits" {
                            return Err(ResponseStatusCode::BadRequest)

                        }
                        return HttpRequestHandler::get_pull_request_logs(pull_request, pull_request_path, path_handler);
                    }
                }
                Err(ResponseStatusCode::BadRequest)
            }
            _ => {
                Err(ResponseStatusCode::BadRequest)
            }
        }
    }

    // TODO change return to http response
    pub fn handle_http(request: Cow<str>, request_type: HttpRequestType, request_url: &str, path_handler: &mut PathHandler) -> Result<SuccessResponse, ResponseStatusCode> {

        let params: Vec<&str> = request_url.split('/').collect();

        if params.len() < 2 || params[1] != "repos" {
            return Err(ResponseStatusCode::BadRequest);
        }

        let repo_name = if params.len() > 2 { Some(params[2]) } else { None };
        let repo = match repo_name {
            Some(repo) => repo,
            _ =>
                return Err(ResponseStatusCode::BadRequest),
        };
        path_handler.set_relative_path(path_handler.get_relative_path(repo));
        match request_type {
            HttpRequestType::GET => {
                HttpRequestHandler::handle_get_request(request, PULL_REQUEST_FILE, request_url, path_handler, repo.to_string())
            },
            HttpRequestType::POST => {
                HttpRequestHandler::add_pull_request(request, PULL_REQUEST_FILE, request_url, path_handler, repo.to_string())
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
        stream.read(&mut buffer)?;

        let request = String::from_utf8_lossy(&buffer[..]);
        let request_type: &str = request.split_whitespace().next().unwrap_or("");
        let request_url: &str = request.split_whitespace().nth(1).unwrap_or_default();
        println!("req_type: -{}-", request_type);
        println!("req_url: -{}-", request_url);
        let http_request_type = HttpRequestType::new(request_type);
        let final_response = match HttpRequestHandler::handle_http(request.clone(), http_request_type, request_url, path_handler) {
            Ok(http_response) => generate_response(http_response.code.to_string(), Some(http_response.body)),
            Err(error) => generate_response(error.to_string(), None),
        };
        println!("Sending response to client: {}", final_response);

        let _ = stream.write(final_response.as_bytes());
        let _ = stream.flush();

        drop(locked_branches_lifetime);

        Ok(())
    }

}

pub fn generate_response(response_code: String, response_body: Option<String>) -> String {

    let current_time: DateTime<Utc> = Utc::now();
    let formatted_time = current_time.format("%a, %d %b %Y %H:%M:%S GMT");

    let mut response = format!("{} {}\r\nDate: {}\r\n", HTTP_VERSION, response_code, formatted_time);

    if let Some(body) = response_body {
        response = format!("{}Content-Type: {}\r\nContent-Length: {}\r\n\r\n{}", response, CONTENT_TYPE, body.len(), body.clone())
    }

    response
}
