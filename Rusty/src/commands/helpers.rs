use std::{collections::HashMap, error::Error, fs, io, io::Read, io::Write, path::Path, fmt};
extern crate crypto;
extern crate libflate;
use std::env;

use crate::commands::{git_commands::Log, structs::{HashObjectCreator, Head}};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use libflate::zlib::{Decoder, Encoder};

use super::git_commands::PathHandler;
use super::structs::{IndexFileEntryState, ObjectType, WorkingDirectory, StagingArea};
use crate::constants::{CONFIG_FILE, CONFLICT_BRANCH_CHANGE, CONFLICT_END, CONFLICT_START, GIT, INDEX_FILE, OBJECT, R_HEADS, R_REMOTES, TREE_FILE_MODE, TREE_SUBTREE_MODE, ZERO_HASH};

/// Returns length of a file's content
pub fn get_file_length(path: &str) -> Result<u64, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

/// Give a file's path it reads it's lines and returns them as a String
pub fn read_file_content(path: &str) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Give a file's path it reads it's lines and returns them as a Vec<u8>
pub fn read_file_content_to_bytes(path: &str) -> Result<Vec<u8>, io::Error> {
    let mut file_content: Vec<u8> = Vec::new();
    let mut file: fs::File = fs::File::open(path)?;
    file.read_to_end(&mut file_content)?;
    Ok(file_content)
}

pub fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
    let mut encoder = Encoder::new(Vec::new())?;

    encoder.write_all(content.as_bytes())?;
    let compressed_data = encoder.finish().into_result()?;

    Ok(compressed_data)
}

pub fn compress_bytes(bytes: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut encoder = Encoder::new(Vec::new())?;

    encoder.write_all(bytes)?;
    let compressed_data = encoder.finish().into_result()?;

    Ok(compressed_data)
}

/// This function takes a `Vec<u8>` containing compressed data, decompresses it using
/// the zlib decoder, and returns the decompressed content as a `String`.
pub fn decompress_file_content(content: Vec<u8>) -> Result<String, io::Error> {
    let mut decompressed_data = String::new();
    let mut decoder = Decoder::new(&content[..])?;
    decoder.read_to_string(&mut decompressed_data)?;
    Ok(decompressed_data)
}

pub fn decompress_file_content_to_bytes(content: Vec<u8>) -> Result<Vec<u8>, io::Error> {
    let mut decompressed_data = Vec::new();
    let mut decoder = Decoder::new(&content[..])?;
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

/// Generates a SHA-1 hash as a hexadecimal string from the provided string
pub fn generate_sha1_string(str: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(str);
    hasher.result_str()
}

/// Updates the index file with a new file path, object hash and status for a specific file.
/// If the file was already contained in the index file, it replaces it.
pub fn update_file_with_hash(
    object_hash: &str,
    new_status: &str,
    file_path: &str,
    path_handler: &PathHandler
) -> io::Result<()> {
    // Read the file into a vector of lines.
    let file_contents = fs::read_to_string(path_handler.get_relative_path(INDEX_FILE))?;

    // Split the file contents into lines.
    let mut lines: Vec<String> = file_contents.lines().map(|s| s.to_string()).collect();

    // Search for the hash in the lines.
    let mut found = false;
    for line in &mut lines {
        if line.starts_with(file_path) {
            found = true;
            // Replace the existing line with the hash and "1".
            *line = format!("{};{};{}", file_path, object_hash, new_status);
            break;
        }
    }

    // If the hash was not found, add a new line.
    if !found {
        lines.push(format!("{};{};{}", file_path, object_hash, new_status));
    }

    // Join the lines back into a single string.
    let updated_contents = lines.join("\n");

    // Write the updated contents back to the file.
    fs::write(path_handler.get_relative_path(INDEX_FILE), updated_contents)?;

    Ok(())
}

/// Removes an object's entry from the index file by its file path.
pub fn remove_object_from_file(file_path: &str) -> io::Result<()> {
    // Read the file into a vector of lines.

    let file_contents = fs::read_to_string(INDEX_FILE)?;

    // Split the file contents into lines.
    let mut lines: Vec<String> = file_contents.lines().map(|s| s.to_string()).collect();

    // Search for the hash in the lines.
    let mut found_index: Option<usize> = None;
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with(file_path) {
            found_index = Some(index);
            break;
        }
    }

    // If the hash was found, remove the line.
    if let Some(index) = found_index {
        lines.remove(index);
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "path did not match any files",
        ));
    }

    // Join the lines back into a single string.
    let updated_contents = lines.join("\n");

    // Write the updated contents back to the file.
    fs::write(INDEX_FILE, updated_contents)?;

    Ok(())
}

pub fn get_all_branches(path_handler: &PathHandler) -> Result<Vec<String>, Box<dyn Error>> {
    let current_branch_path = &Head::get_current_branch_path(path_handler)?;

    // Extract the directory path from the file path
    let dir_path = Path::new(&current_branch_path)
        .parent()
        .ok_or("Failed to get parent directory")?;

    let entries = fs::read_dir(path_handler.get_relative_path(&dir_path.to_string_lossy()))?;

    // Iterate over the entries
    let mut branches: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry?;

        let dir_path_without_git = dir_path.strip_prefix(".git/").unwrap_or(dir_path);

        let file_name = dir_path_without_git
            .join(entry.file_name())
            .to_string_lossy()
            .into_owned();

        let file_content = fs::read_to_string(entry.path())?;

        // Combine content and filename
        let branch = format!("{} {}\n", file_content, file_name);
        branches.push(branch);
    }

    Ok(branches)
}

pub fn get_remote_url(name: &str) -> Result<String, Box<dyn Error>> {
    let config_content = read_file_content(CONFIG_FILE)?;
    let current_remote_line = format!("[remote '{}']", name);
    let mut in_remote = false;

    for line in config_content.lines() {
        if line == current_remote_line.as_str() {
            in_remote = true;
        } else if in_remote {
            let parts: Vec<&str> = line.split(' ').collect();
            let url = parts.last().unwrap_or(&"");
            return Ok(url.to_string());
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "No remote found.",
    )))
}

pub fn generate_sha1_string_from_bytes(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.input(data);
    hasher.result_str()
}

pub fn read_object_to_bytes(hash: String, path_handler: &PathHandler) -> Result<(ObjectType, Vec<u8>, String), Box<dyn Error>> {
    let mut file = fs::File::open(path_handler.get_relative_path(&get_object_path(&hash)))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let file_data = decompress_file_content_to_bytes(buffer)?;

    let split_content: Vec<Vec<u8>> = file_data
        .splitn(2, |&c| c == 0)
        .map(|slice| slice.to_vec())
        .collect();

    let object_header: Vec<String> = String::from_utf8_lossy(&split_content[0])
        .to_string()
        .split(' ')
        .map(String::from)
        .collect();
    let object_type = ObjectType::new(&object_header[0]).ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Failed to determine object type",
    ))?;
    let object_size = if object_header.len() >= 2 {
        object_header[1].clone()
    } else {
        String::new()
    };

    Ok((object_type, split_content[1].clone(), object_size))
}

pub fn read_object_to_string(hash: String, path_handler: &PathHandler) -> Result<(ObjectType, String, String), Box<dyn Error>> {
    let (object_type, file_content, object_size) = read_object_to_bytes(hash, path_handler)?;

    let content_to_string = String::from_utf8_lossy(&file_content).to_string();

    Ok((object_type, content_to_string, object_size))
}

pub fn get_remote_tracking_branches(path_handler: &PathHandler) -> Result<HashMap<String, (String, String)>, Box<dyn Error>> {
    // Read the content of the Git configuration file
    let config_content = read_file_content(&path_handler.get_relative_path(CONFIG_FILE))?;

    // Initialize a HashMap to store branch names and their corresponding remotes
    let mut branches_and_remotes = HashMap::new();

    // Parse the INI-like configuration content manually
    let mut lines = config_content.lines().peekable();
    while let Some(line) = lines.next() {
        // Check if the line represents a section header
        if line.starts_with("[branch ") {
            let branch_name = line.trim_start_matches("[branch ").trim_end_matches(']');
            // Extract remote and merge values
            let mut remote = None;
            let mut merge = None;

            for next_line in lines.by_ref() {
                if next_line.starts_with("remote = ") {
                    remote = Some(next_line.trim_start_matches("remote = ").to_string());
                } else if next_line.starts_with("merge = ") {
                    merge = Some(
                        next_line
                            .trim_start_matches("merge = refs/heads/")
                            .to_string(),
                    );
                } else if next_line.starts_with('[') {
                    // End of the current section
                    break;
                }
            }

            // If both remote and merge values are present, store in the HashMap
            if let (Some(remote), Some(merge)) = (remote, merge) {
                branches_and_remotes.insert(branch_name.to_string(), (remote, merge));
            }
        }
    }
    Ok(branches_and_remotes)
}

/// Reads remote branches from remotes directory and returns a tuple with (branch_name, last_commit_hash)
pub fn get_remote_branches(remote_name: &str, path_handler: &PathHandler) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut branches_to_update: Vec<(String, String)> = Vec::new();
    
    let remote_branches_path = format!("{}/{}", R_REMOTES, remote_name);
    let remote_branches = fs::read_dir(path_handler.get_relative_path(&remote_branches_path))?;

    for branch in remote_branches {
        let branch = branch?;
        let branch_name = branch.file_name().to_string_lossy().to_string();
        let branch_path = branch.path();

        let branch_hash = read_file_content(&path_handler.get_relative_path(branch_path.to_str().ok_or("")?))?;
        branches_to_update.push((branch_name, branch_hash))
    }
    Ok(branches_to_update)
}

pub fn update_branches(branches: Vec<(String, String)>, path_handler: &PathHandler) -> Result<(), Box<dyn Error>>{
    for (branch_name, hash) in branches {
        let branch_file = format!("{}/{}", R_HEADS, branch_name);
        let mut branch_file = fs::File::create(path_handler.get_relative_path(&branch_file))?;
        branch_file.write_all(hash.as_bytes())?;
    }

    Ok(())
}

pub fn update_local_branch_with_commit(
    remote_name: &str,
    branch_name: &str,
    remote_hash: &str,
    path_handler: &PathHandler
) -> Result<(), Box<dyn Error>> {
    let config_content = read_file_content(&path_handler.get_relative_path(CONFIG_FILE))?;

    let branch_header = format!("[branch '{}']", branch_name);
    let mut lines = config_content.lines().peekable();
    while let Some(line) = lines.next() {
        if line == branch_header {
            let mut remote = None;
            for next_line in lines.by_ref() {
                if next_line.starts_with("remote = ") {
                    remote = Some(next_line.trim_start_matches("remote = ").to_string());
                } else if next_line.starts_with('[') {
                    break;
                }
            }
            if let Some(remote) = remote {
                if remote == remote_name {
                    let _ = update_branch_hash(branch_name, remote_hash, path_handler);
                }
            }
        }
    }
    Ok(())
}

/// Updates the commit which the specified branch points to
pub fn update_branch_hash(branch_name: &str, new_commit_hash: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create(path_handler.get_relative_path(&get_branch_path(
        branch_name,
    )))?;
    file.write_all(new_commit_hash.as_bytes())?;
    Ok(())
}

/// Returns the commit the specified branch points to
pub fn get_branch_last_commit(branch_path: &str, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
    
    let mut file: fs::File = fs::File::open(path_handler.get_relative_path(branch_path))?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn get_branch_path(branch_name: &str) -> String {
    format!("{}/{}", R_HEADS, branch_name)
}

pub fn get_object_path(object_hash: &str) -> String {
    format!(
        "{}/{}/{}",
        OBJECT,
        &object_hash[..2].to_string(),
        &object_hash[2..].to_string()
    )
}

/// Goes through the logs of the merging branch and head branch looking
/// for its common ancestor commit. If found it returns said commit hash.
/// If not, it return an empty string
pub fn find_common_ancestor_commit(
    current_branch_commit: &str,
    merging_branch: &str,
    path_handler: &PathHandler
) -> Result<String, Box<dyn Error>> {
    let mut current_branch_log = Vec::new();
    let _ = Log::generate_log_entries(&mut current_branch_log, current_branch_commit.to_string(), path_handler);

    let mut merging_branch_log = Vec::new();
    let _ = Log::generate_log_entries(&mut merging_branch_log, merging_branch.to_string(), path_handler);

    for (commit, _message) in merging_branch_log {
        if current_branch_log.contains(&(commit.clone(), _message)) {
            return Ok(commit);
        }
    }

    Ok(String::new())
}

pub fn ancestor_commit_exists(
    current_commit_hash: &str,
    merging_commit_hash: &str,
    path_handler: &PathHandler
) -> Result<bool, Box<dyn Error>> {
    let mut merging_branch_log = Vec::new();
    if current_commit_hash.is_empty() {
        return Ok(true);
    }

    let _ = Log::generate_log_entries(&mut merging_branch_log, merging_commit_hash.to_string(), path_handler);
    for (commit, _message) in merging_branch_log {
        if commit == current_commit_hash {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Given a commit's hash it accesses its file and returns the hash of its associated
/// tree object.
pub fn get_commit_tree(commit_hash: &str, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
    let get_obj_path = get_object_path(commit_hash);
    let get_obj_path_relative = path_handler.get_relative_path(&get_obj_path);

    let read_file_content_to_bytes = read_file_content_to_bytes(&get_obj_path_relative)?;
    let decompressed_data = decompress_file_content(read_file_content_to_bytes)?;

    let commit_file_content: Vec<String> =
        decompressed_data.split('\0').map(String::from).collect();
    let commit_file_lines: Vec<String> = commit_file_content[1]
        .lines()
        .map(|s| s.to_string())
        .collect();

    let tree_split_line: Vec<String> = commit_file_lines[0]
        .split_whitespace()
        .map(String::from)
        .collect();

    let tree_hash_trimmed = &tree_split_line[1];

    Ok(tree_hash_trimmed.to_string())
}

/// Checks if the file in the given path exists and returns true or false
pub fn check_if_file_exists(file_path: &str, path_handler: &PathHandler) -> bool {
    if let Ok(metadata) = fs::metadata(path_handler.get_relative_path(file_path)) {
        if metadata.is_file() {
            return true;
        }
    }
    false
}

/// Checks if the directory in the given path exists and returns true or false
pub fn check_if_directory_exists(dir_path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(dir_path) {
        if metadata.is_dir() {
            return true;
        }
    }
    false
}

pub fn hex_string_to_bytes(bytes: &[u8]) -> String {
    let mut hash: String = String::new();
    for byte in bytes {
        hash.push_str(&format!("{:02x}", byte));
    }

    hash
}

type TreeContent = (String, String, String);

pub fn read_tree_content(tree_hash: &str, path_handler: &PathHandler) -> Result<Vec<TreeContent>, Box<dyn Error>> {
    let compressed_content =
        read_file_content_to_bytes(&path_handler.get_relative_path(&get_object_path(tree_hash)))?;
    let tree_content = decompress_file_content_to_bytes(compressed_content)?;
    let split_content: Vec<Vec<u8>> = tree_content
        .splitn(2, |&c| c == 0)
        .map(|slice| slice.to_vec())
        .collect();

    let mut divided_content = Vec::new();

    let mut substrings: Vec<Vec<u8>> = split_content[1]
        .split(|&c| c == 0)
        .map(|slice| slice.to_vec())
        .collect();

    let tree_data: Vec<Vec<u8>> = substrings[0]
        .split(|&c| c == 32)
        .map(|slice| slice.to_vec())
        .collect();

    let mut file_mode = String::from_utf8_lossy(&tree_data[0]).to_string();
    let mut file_name = String::from_utf8_lossy(&tree_data[1]).to_string();

    substrings.remove(0);


    for substring in &substrings {
        let processed_bytes = &substring[..20];

        let hash_string = hex_string_to_bytes(processed_bytes);

        divided_content.push((file_mode.clone(), file_name.clone(), hash_string));

        if substring.len() > 20 {
            let tree_entry_data = String::from_utf8_lossy(&substring[20..]).to_string();
            
            let split_entry: Vec<String> = tree_entry_data
                .split_whitespace()
                .map(String::from)
                .collect();
            file_mode = split_entry[0].clone();
            file_name = split_entry[1].clone();
        }
    }

    Ok(divided_content)
}

pub fn convert_hash_to_decimal_bytes(hash: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut decimal_hash = Vec::new();
    for chunk in hash.chars().collect::<Vec<char>>().chunks(2) {
        let chunk_str: String = chunk.iter().collect();
        let result = u8::from_str_radix(&chunk_str, 16)?;
        decimal_hash.push(result);
    }

    Ok(decimal_hash)
}

pub fn validate_ref_update_request(
    prev_remote_hash: &str,
    _new_remote_hash: &str,
    branch_ref: &str,
    path_handler: &PathHandler
) -> Result<(), Box<dyn Error>> {
    
    let branch_path = format!(".git/{}", branch_ref);
    
    if check_if_file_exists(&branch_path, path_handler) {
        /*if prev_remote_hash == ZERO_HASH {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Trying to initialize existing ref",
            )));
        }
        if get_branch_last_commit(&PathHandler::get_relative_path(&branch_path))?
            != prev_remote_hash
        {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: New hash is different from ref's current hash",
            )));
        }*/
    } else if prev_remote_hash != ZERO_HASH {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Error: Ref was not found",
        )));
    }

    Ok(())
}

pub fn update_hash_for_refs(
    refs_to_update: Vec<(String, String, String)>, path_handler: &PathHandler
) -> Result<(), Box<dyn Error>> {
    for (_, new_remote_hash, branch_ref) in refs_to_update {
        let ref_path = format!(".git/{}", branch_ref);
        let mut file = fs::File::create(path_handler.get_relative_path(&ref_path))?;
        file.write_all(new_remote_hash.as_bytes())?
    }
    Ok(())
}

pub fn find_modified_files(ancestor_working_tree: HashMap<String, String>, working_tree_to_compare:  HashMap<String, String>) -> HashMap<String, String> {
    let mut modified_files: HashMap<String, String> = HashMap::new();
    for (file_name, file_hash) in working_tree_to_compare {
        if let Some(ancestor_hash) = ancestor_working_tree.get(&file_name) {
            if *ancestor_hash != file_hash {
                modified_files.insert(file_name, file_hash);
            }
        } else {
            modified_files.insert(file_name, file_hash);
        }
    }
    modified_files
}

/// Given two working trees stored in HashMaps, it goes through their content and check if any conflict is found when merging.
/// Returning a HashMap that contains the files without conflict name's and hashes;
pub fn find_files_without_conflict(ancestor_working_tree: HashMap<String, String>, current_modified_files: HashMap<String, String>, mut merging_modified_files:  HashMap<String, String>, path_handler: &PathHandler) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut files_without_conflict: HashMap<String, String> = HashMap::new();
    let mut files_with_conflict: Vec<(String, String)> = Vec::new(); // tal vez esto ni hace falta si los voy printeando

    for (file_name, file_hash) in current_modified_files {
        if let Some(merging_hash) = merging_modified_files.remove(&file_name) {
            if merging_hash != file_hash {
                let merged_file = find_conflict_in_file(file_name.clone(), ancestor_working_tree.get(&file_name).ok_or("Ancestor file not found")?.to_string(), file_hash.clone(), merging_hash, path_handler)?;
                if merged_file.is_empty() {
                    println!("CONFLICT: Merge conflict in {}", file_name);
                    files_with_conflict.push((file_name.clone(), file_hash));
                } else {
                    files_without_conflict.insert(file_name, merged_file);
                }
            } else {
                files_without_conflict.insert(file_name, file_hash);
            }
        } else {
            files_without_conflict.insert(file_name, file_hash);
        }
    }

    for (file_name, file_hash) in merging_modified_files {
        files_without_conflict.insert(file_name, file_hash);
    }

    StagingArea::new().change_index_file(files_without_conflict.clone(), files_with_conflict.clone(), path_handler)?;

    if !files_with_conflict.is_empty() {
        println!("Automatic merge failed; fix conflicts and then commit the result");
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Merge failed",
        )))
    }

    Ok(files_without_conflict)
}

pub fn find_conflict_in_file(file_name: String, ancestor_hash: String, first_object_hash: String, second_object_hash: String, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
    let changes_in_first_object = find_changes_in_file(file_name.clone(), ancestor_hash.clone(), first_object_hash.clone(), path_handler)?;
    let changes_in_second_object = find_changes_in_file(file_name.clone(), ancestor_hash, second_object_hash.clone(), path_handler)?;
    let max_len = changes_in_first_object.len().max(changes_in_second_object.len());

    let mut final_merged_content: Vec<String> = Vec::new();
    let mut conflict_was_found = false;

    let mut first_object_conflict_lines: Vec<String> = Vec::new(); // Store consecutive conflicting lines
    let mut second_object_conflict_lines: Vec<String> = Vec::new();

    for i in 0..max_len + 1 {
        match (changes_in_first_object.get(i), changes_in_second_object.get(i)) {
            (Some(LineChange::Modified(_, _)), Some(LineChange::Modified(_, _)))
            | (Some(LineChange::Modified(_, _)), Some(LineChange::Deleted(_, _)))
            | (Some(LineChange::Deleted(_, _)), Some(LineChange::Modified(_, _))) => {
                conflict_was_found = true;

                if let Some(LineChange::Modified(_, line)) = changes_in_first_object.get(i) {
                    first_object_conflict_lines.push(line.to_string());
                }
                if let Some(LineChange::Modified(_, line)) = changes_in_second_object.get(i) {
                    second_object_conflict_lines.push(line.to_string());
                }
            }
            (Some(LineChange::Added(_, line1)), Some(LineChange::Added(_, line2))) => {
                final_merged_content.push(line1.to_string());
                final_merged_content.push(line2.to_string());
            }
            (Some(LineChange::Modified(_, line)), _)
            | (_, Some(LineChange::Modified(_, line)))
            | (_, Some(LineChange::Same(_, line)))
            | (Some(LineChange::Same(_, line)), _) => {
                if !first_object_conflict_lines.is_empty() {
                    final_merged_content.extend(generate_marked_conflict_lines(first_object_conflict_lines.clone(), second_object_conflict_lines.clone()));
                    first_object_conflict_lines.clear();
                    second_object_conflict_lines.clear();
                }
                final_merged_content.push(line.to_string());
            }
            _ => {
                if !first_object_conflict_lines.is_empty() {
                    final_merged_content.extend(generate_marked_conflict_lines(first_object_conflict_lines.clone(), second_object_conflict_lines.clone()));
                    first_object_conflict_lines.clear();
                    second_object_conflict_lines.clear();
                }
            }
        }
    }
    let merged_content_joined = final_merged_content.join("\n");
    
    if conflict_was_found {
        let mut file_with_conflicts = fs::File::create(path_handler.get_relative_path(&file_name))?;
        file_with_conflicts.write_all(merged_content_joined.as_bytes())?;
        return Ok(String::new());
    }
    
    let new_object_hash = HashObjectCreator::write_object_file(merged_content_joined.clone(), ObjectType::Blob, merged_content_joined.len() as u64, path_handler)?;
    Ok(new_object_hash)
}

fn generate_marked_conflict_lines(first_branch_conflict_lines: Vec<String>, second_branch_conflict_lines: Vec<String>) -> Vec<String> {
    let mut conflict_lines: Vec<String> = Vec::new();
    
    conflict_lines.push(CONFLICT_START.to_string());
    conflict_lines.extend(first_branch_conflict_lines);
    conflict_lines.push(CONFLICT_BRANCH_CHANGE.to_string());
    conflict_lines.extend(second_branch_conflict_lines);
    conflict_lines.push(CONFLICT_END.to_string());

    conflict_lines
}

pub fn find_changes_in_file(_file_name: String, ancestor_hash: String, branch_hash: String, path_handler: &PathHandler) -> Result<Vec<LineChange>, Box<dyn Error>> {
    let (_, ancestor_object_content, _) = read_object_to_string(ancestor_hash, path_handler)?;
    let (_, changed_object_content, _) = read_object_to_string(branch_hash, path_handler)?;

    let ancestor_object_lines: Vec<String> = ancestor_object_content.lines().map(|line| line.to_string()).collect();
    let changed_object_lines: Vec<String> = changed_object_content.lines().map(|line| line.to_string()).collect();

    let mut changes = Vec::new();
    let max_len = ancestor_object_lines.len().max(changed_object_lines.len());

    for i in 0..max_len {
        match (ancestor_object_lines.get(i), changed_object_lines.get(i)) {
            (Some(line1), Some(line2)) if line1 == line2 => {
                changes.push(LineChange::Same(i + 1, line1.to_string()));
            }
            (Some(_line1), Some(line2)) => {
                changes.push(LineChange::Modified(i + 1, line2.to_string()));
            }
            (Some(line1), None) => {
                changes.push(LineChange::Deleted(i + 1, line1.to_string()));
            }
            (None, Some(line2)) => {
                changes.push(LineChange::Added(i + 1, line2.to_string()));
            }
            _ => unreachable!(), // We should never reach this case
        }
    }
    Ok(changes)
}

#[derive(Debug)]
pub enum LineChange {
    Same(usize, String),
    Modified(usize, String),
    Added(usize, String),
    Deleted(usize, String),
}

impl fmt::Display for LineChange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            LineChange::Added(_, line) => format!("Added: {}", line),
            LineChange::Modified(_, line) => format!("Modified: {}", line),
            LineChange::Same(_, line) => format!("Same: {}", line),
            LineChange::Deleted(_, line) => format!("Deleted: {}", line),
        };
        write!(f, "{}", string)
    }
}
/// This function goes through the tree object associated to a commit object and
/// adds all of the files in its working tree into a HashMap, where the file name (path)
/// is the key and its corresponding object hash is the value stored.
pub fn reconstruct_working_tree(commit_hash: String, path_handler: &PathHandler) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let tree_content = read_tree_content(&get_commit_tree(&commit_hash, path_handler)?, path_handler)?;
    let mut working_tree: HashMap<String, String> = HashMap::new();

    for (file_mode, file_name, file_hash) in tree_content {
        match file_mode.as_str() {
            TREE_FILE_MODE => {
                working_tree.insert(file_name, file_hash);
            }
            TREE_SUBTREE_MODE => {
                working_tree.extend(reconstruct_working_tree(file_hash, path_handler)?);
            }
            _ => {}
        }
    }
    Ok(working_tree)
}

/// Receives a repo name and return a result indicating if the repo already exists or not
pub fn check_if_repo_exists(repo_name: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
    if repo_name.is_empty() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Error: repo name can't be blank, it does not exist.",
        )))
    }
    let string_full_path = path_handler.get_relative_path(repo_name);
    let full_path = Path::new(&string_full_path);

    // Check if the given directory exists
    if full_path.is_dir() {
        println!("Directory '{:?}' exists.", full_path);
        println!("checking if it's a git repo...");
        let git_dir = full_path.join(GIT);
        if git_dir.is_dir() {
            println!("Subdirectory '.git' exists inside '{:?}'.", git_dir);
            Ok(())
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Subdirectory '.git' does not exist.",
            )))
        }
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Directory does not exist.",
        )))
    }
}

/// Receives a branch name and a repo name, returns a result indicating if the branch already exists in the repo or not
pub fn check_if_branch_belongs_to_repo(branch_name: &str, _repo_name: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
    let branch_path = get_branch_path(branch_name);

    if !check_if_file_exists(&branch_path, path_handler) {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Error: Specified branch does not exist in this repo.",
        )))
    }
    Ok(())
}
/// Receives a branch name and return a result indicating if the branch already exists or not
pub fn get_client_current_working_repo() -> Result<String, Box<dyn Error>> {
    if let Ok(current_dir) = env::current_dir() {
        if let Some(parent) = current_dir.file_name() {
            if let Some(parent_str) = parent.to_str() {
                return Ok(parent_str.to_string());
            }
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "Error: current client working repo couldn't be found.",
    )))
}

/// Receives a branch name and return a result indicating if the branch already exists or not
pub fn check_if_branch_exists(branch_name: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
    let branch_path = get_branch_path(branch_name);
    if !check_if_file_exists(&branch_path, path_handler) {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Error: Specified branch does not exist.",
        )))
    }
    Ok(())
}

/// Given two commits it finds which files can be merged without generating a conflict. 
/// It then cleans the working directory of the old and not merged files and updates the 
/// index file to the new working tree.
pub fn determine_new_working_tree(commit_merging_into: String, commit_to_merge: String, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
    let ancestor_commit = find_common_ancestor_commit(&commit_merging_into, &commit_to_merge, path_handler)?;
    let ancestor_working_tree = reconstruct_working_tree(ancestor_commit, path_handler)?;
    let current_working_tree = reconstruct_working_tree(commit_merging_into, path_handler)?;
    let merging_working_tree = reconstruct_working_tree(commit_to_merge.clone(), path_handler)?;
    let _files_without_conflict = find_files_without_conflict(ancestor_working_tree, current_working_tree, merging_working_tree, path_handler)?;

    WorkingDirectory::clean_working_directory(path_handler)?;

    Ok(())
}

// pub fn change_commit_object_field(commit_object_to_change: String, field_to_change: ObjectType, new_hash: String) -> Result<(), Box<dyn Error>> {
//     println!("commit: {}", commit_object_to_change);
//     let commit_file_content = read_file_content(&get_object_path(&commit_object_to_change))?;
//     println!("content: {}", commit_file_content);
//     let split_commit_content: Vec<String> = commit_file_content.split('\n').map(String::from).collect();
//     let mut new_file_content: Vec<String> = Vec::new();

//     for line in split_commit_content {
//         let split_line: Vec<String> = line.split(' ').map(String::from).collect();
//         if split_line[1] == field_to_change.to_string() {
//             let new_line = format!("{} {}", field_to_change, new_hash);
//             new_file_content.push(new_line);
//         } else {
//             new_file_content.push(line)
//         }
//     }

//     HashObjectCreator::write_object_file(content, obj_type, file_len, path_handler);

//     let mut commit_file = fs::File::create(get_object_path(&commit_object_to_change))?;
//     commit_file.write_all(new_file_content.join("\n").as_bytes())?;

//     Ok(())
// }

pub fn create_merged_working_tree(head_commit: String, merging_commit: String, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
    StagingArea::new().stage_index_file(path_handler)?;
    let new_commit_hash = HashObjectCreator::create_commit_object(None, vec![head_commit, merging_commit], path_handler)?;
    
    update_branch_hash(&Head::get_current_branch_name(path_handler)?, &new_commit_hash, path_handler)?;

    let commit_tree = get_commit_tree(&new_commit_hash, path_handler)?;
    WorkingDirectory::update_working_directory_to(&commit_tree, path_handler)?;

    Ok(new_commit_hash)
}

pub fn check_if_conflict_has_been_solved(path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
    let index_file_content = read_file_content(&path_handler.get_relative_path(INDEX_FILE))?;

    let index_lines: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();

    for line in index_lines {
        let split_line: Vec<&str> = line.split(';').collect();
        if split_line[2] == IndexFileEntryState::Conflicted.to_string() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Conflict in file has not been solved.",
            ))) 
        }
    }
    Ok(())
}

pub const RELATIVE_PATH: &str = "RELATIVE_PATH";
/* pub const RELATIVE_PATH: &str = "RELATIVE_PATH";
#[cfg(test)]
mod tests {
    use crate::commands::commands::{Init, Command, Branch};

    use super::*;
    use std::{fs, env};
    use tempfile::tempdir;

    fn common_setup() -> (tempfile::TempDir, String) {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        // Set the environment variable for the relative path
        env::set_var(RELATIVE_PATH, &temp_path);

        // Create and execute the Init command
        let init_command = Init::new();
        let result = init_command.execute(None);

        // Check if the Init command was successful
        assert!(result.is_ok(), "Init command failed: {:?}", result);

        (temp_dir, temp_path)
    }

    #[test]
    fn test_get_current_branch_path() {
        // Common setup: create a temporary directory and initialize a Git repository
        let (_temp_dir, _temp_path) = common_setup();

        // Create a sample branch name
        let branch_name = "main";

        // Create and execute the Branch command to set the initial branch
        let branch_command = Branch::new();
        let result = branch_command.execute(Some((&[branch_name]).to_vec()));
        assert!(result.is_ok(), "Branch command failed: {:?}", result);

        // Call the function to get the current branch path
        let current_branch_path = get_current_branch_path().expect("Failed to get current branch path");

        // Check if the current branch path matches the expected path
        let expected_branch_path = format!(".git/refs/heads/{}", branch_name);
        assert_eq!(current_branch_path, expected_branch_path);
    }


    #[test]
    fn test_get_file_length() {
        // Create a temporary file with some content
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "Test content").expect("Failed to write to file");

        // Call the function to get the file length
        let file_length = get_file_length(file_path.to_str().unwrap()).unwrap();

        // Check if the file length is as expected
        assert_eq!(file_length, 12);
    }

    #[test]
    fn test_read_file_content() {
        // Create a temporary file with some content
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "Test content").expect("Failed to write to file");

        // Call the function to read file content
        let content = read_file_content(file_path.to_str().unwrap()).unwrap();

        // Check if the content is as expected
        assert_eq!(content, "Test content");
    }

    #[test]
    fn test_read_file_content_to_bytes() {
        // Create a temporary file with some content
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "Test content").expect("Failed to write to file");

        // Call the function to read file content to bytes
        let content_bytes = read_file_content_to_bytes(file_path.to_str().unwrap()).unwrap();

        // Check if the content bytes are as expected
        assert_eq!(content_bytes, b"Test content");
    }
    #[test]
    fn test_compress_and_decompress_content() {
        let original_content = "This is a test content.";
        let compressed_content = compress_content(original_content).unwrap();
        let decompressed_content = decompress_file_content(compressed_content).unwrap();
        assert_eq!(original_content, decompressed_content);
    }

    #[test]
    fn test_generate_sha1_string() {
        let input_str = "Hello, world!";
        let hash = generate_sha1_string(input_str);
        // TODO: Add assertions based on known hash values
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn test_create_new_branch() {
        // Common setup: create a temporary directory and initialize a Git repository
        let (_temp_dir, _temp_path) = common_setup();

        // Create a new branch
        let mut head = Head::new();
        let branch_name = "test_branch";
        create_new_branch(branch_name, &mut head).expect("Failed to create new branch");

        // Check if the branch file was created
        let branch_file_path = format!(".git/refs/heads/{}", branch_name);
        assert!(Path::new(&PathHandler::get_relative_path(&branch_file_path)).exists(), "Branch file not created");

        // Check if the Head state was updated
        let current_branch = head.get_current_branch().expect("Failed to get current branch");
        assert_eq!(current_branch, branch_name, "Head state not updated");
    }

}
 */
