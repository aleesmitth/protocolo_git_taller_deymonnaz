use crate::commands::git_commands::Command;
use crate::commands::git_commands::PackObjects;
use crate::commands::git_commands::PathHandler;
use crate::commands::git_commands::UnpackObjects;
use crate::commands::helpers;
use crate::commands::protocol_utils;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::{Mutex, Arc, Condvar};
use std::thread;
use std::time::Duration;
use crate::constants::{REQUEST_LENGTH_CERO, REQUEST_DELIMITER_DONE, WANT_REQUEST, NAK_RESPONSE, UNPACK_CONFIRMATION, ALL_BRANCHES_LOCK};

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

    pub fn lock_branch(branch: &str, locked_branches: &Arc<(Mutex<HashSet<String>>, Condvar)>, all_branches_locked_by_me: bool) -> Result<(), Box<dyn Error>> {// Extract the Mutex and Condvar from the Arc
        // Extract the Mutex and Condvar from the Arc
        let (lock, cvar) = &**locked_branches;

        // Acquire the lock before checking or modifying the set of locked branches
        let mut locked_branches = match lock.lock() {
            Ok(lock) => lock,
            Err(err) => {
                // Handle the error, according to doc. Previous holder of mutex panicked
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    err.to_string(),
                )));
            }
        };

        // Wait for the branch to be available
        while locked_branches.contains(branch) || (locked_branches.contains(ALL_BRANCHES_LOCK) && !all_branches_locked_by_me) {
            println!("Branch '{}' locked going to sleep, wait for CondVar notification. Lock of HashMap released", branch);
            // Release the lock before waiting and re-acquire it after waking up
            locked_branches = match cvar.wait(locked_branches) {
                Ok(guard) => guard,
                Err(err) => {
                    // Handle the error, potentially logging or returning an error response
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        err.to_string(),
                    )));
                }
            };
            // Perform your branch-specific operations here
            println!("CondVar notification received, lock reacquired");
        }

        // Branch is not locked, so lock it
        locked_branches.insert(branch.to_string());

        // Release the lock outside the loop
        drop(locked_branches);

        println!("Branch inserted in HashMap: '{}'. Lock of HashMap released", branch);
        Ok(())
    }

    pub fn unlock_branch(branch: &str, locked_branches: &Arc<(Mutex<HashSet<String>>, Condvar)>) -> Result<(), Box<dyn Error>> {
        // Extract the Mutex and Condvar from the Arc
        let (lock, cvar) = &**locked_branches;

        // Acquire the lock before checking or modifying the set of locked branches
        let mut locked_branches = match lock.lock() {
            Ok(lock) => lock,
            Err(err) => {
                // Handle the error, according to doc. Previous holder of mutex panicked
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    err.to_string(),
                )));
            }
        };

        // Remove the branch if it exists
        if locked_branches.remove(branch) {
            // Notify other waiting threads that the condition (branch availability) has changed
            cvar.notify_all();
        }

        // Release the lock
        drop(locked_branches);

        // Perform your branch-specific operations here
        println!("Branch removed from HashMap: '{}'. Lock of HashMap released", branch);

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

        locked_branches_lifetime.lock_branch(ALL_BRANCHES_LOCK, locked_branches, false)?;
        
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
            locked_branches_lifetime.lock_branch(branch, locked_branches, true)?;
        }

        locked_branches_lifetime.unlock_branch(ALL_BRANCHES_LOCK, locked_branches)?;

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

        locked_branches_lifetime.lock_branch(ALL_BRANCHES_LOCK, locked_branches, false)?;      

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

                locked_branches_lifetime.lock_branch(branch_name, locked_branches, true)?;

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

        locked_branches_lifetime.unlock_branch(ALL_BRANCHES_LOCK, locked_branches)?;

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


// Define a struct to represent the locked branches
struct LockedBranches<'a> {
    locked_branches: &'a Arc<(Mutex<HashSet<String>>, std::sync::Condvar)>,
    current_branch_locked_branches: HashSet<String>,
}

impl<'a> LockedBranches<'a> {
    fn new(locked_branches: &'a Arc<(Mutex<HashSet<String>>, std::sync::Condvar)>) -> Self {
        LockedBranches { 
            locked_branches,
            current_branch_locked_branches: HashSet::new(),
        }
    }

    fn lock_branch(&mut self, branch_to_lock: &str, locked_branches: &'a Arc<(Mutex<HashSet<String>>, std::sync::Condvar)>, should_extend: bool) -> Result<(), Box<dyn Error>> {
        println!("locking branch: {}", branch_to_lock);
        ServerProtocol::lock_branch(branch_to_lock, locked_branches, should_extend)?;
        self.current_branch_locked_branches.insert(branch_to_lock.to_string());

        Ok(())
    }

    fn unlock_branch(&mut self, branch_to_unlock: &str, locked_branches: &'a Arc<(Mutex<HashSet<String>>, std::sync::Condvar)>) -> Result<(), Box<dyn Error>> {
        println!("unlocking branch: {}", branch_to_unlock);
        ServerProtocol::unlock_branch(&branch_to_unlock, self.locked_branches)?;
        self.current_branch_locked_branches.remove(branch_to_unlock);

        Ok(())
    }
}

// Implement Drop trait for automatic unlocking
impl<'a> Drop for LockedBranches<'a> {
    fn drop(&mut self) {
        println!("dropping branches");

        for locked_branch in &self.current_branch_locked_branches {
            if ServerProtocol::unlock_branch(&locked_branch, self.locked_branches).is_err() {
                println!("Error unlocking branch. Please restart server.")
            }
        }
    }
}
