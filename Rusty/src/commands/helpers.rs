use std::{
    collections::HashMap, error::Error, fs, io, io::Read, io::Write, path::Path,
};
extern crate crypto;
extern crate libflate;

use crate::commands::structs::Head;
use crate::commands::commands::Log;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use libflate::zlib::{Decoder, Encoder};

use super::{commands::PathHandler, structs::ObjectType};

const OBJECT: &str = ".git/objects";
const R_HEADS: &str = ".git/refs/heads";
const HEAD_FILE: &str = ".git/HEAD";
const DEFAULT_BRANCH_NAME: &str = "main";
const INDEX_FILE: &str = ".git/index";
const CONFIG_FILE: &str = ".git/config";
//const R_REMOTES: &str = ".git/refs/remotes";

/// Retrieves the path to the current branch from the Git HEAD file.
pub fn get_current_branch_path() -> Result<String, Box<dyn Error>> {
    let head_file_content = read_file_content(&PathHandler::get_relative_path(HEAD_FILE))?;
    let split_head_content: Vec<&str> = head_file_content.split(" ").collect();
    if let Some(branch_path) = split_head_content.get(1) {
        let full_branch_path: String = format!(".git/{}", branch_path);
        return Ok(full_branch_path);
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "Eror reading branch path",
    )))
}

/// Returns head commit
pub fn get_head_commit() -> Result<String, Box<dyn Error>> {
    let mut file = fs::File::open(get_current_branch_path()?)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

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

/// Given a file's content it compresses it using an encoder from the libflate external crate and
/// returns a Vec<u8> containing the encoded content
// pub fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
//     let mut encoder = Encoder::new(Vec::new())?;
//     encoder.write_all(content.as_bytes())?;
//     encoder.finish().into_result()
// }
pub fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
    // Crea un nuevo `Encoder` y un vector para almacenar los datos comprimidos.
    let mut encoder = Encoder::new(Vec::new())?;

    // Escribe el contenido (en bytes) en el `Encoder`.
    encoder.write_all(content.as_bytes())?;

    // Finaliza la compresión y obtiene el resultado comprimido.
    let compressed_data = encoder.finish().into_result()?;

    // Devuelve el resultado comprimido.
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

/// Generates a SHA-1 hash as a hexadecimal string from the provided string
pub fn generate_sha1_string(str: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(str);
    hasher.result_str()
}

/// Creates a new branch with the specified name. Creates branch file.
pub fn create_new_branch(branch_name: &str, head: &mut Head) -> Result<(), Box<dyn Error>> {
    let branch_path = format!("{}/{}", R_HEADS, branch_name);

    let previous_branch_path = get_current_branch_path()?;
    let last_commit_hash =
        read_file_content(&PathHandler::get_relative_path(&previous_branch_path))?;
    let mut branch_file = fs::File::create(&PathHandler::get_relative_path(&branch_path))?;

    if branch_name == DEFAULT_BRANCH_NAME {
        write!(branch_file, "")?;
    } else {
        write!(branch_file, "{}", last_commit_hash)?;
    }
    head.add_branch(branch_name);

    Ok(())
}

/// Updates the index file with a new file path, object hash and status for a specific file.
/// If the file was already contained in the index file, it replaces it.
pub fn update_file_with_hash(
    object_hash: &str,
    new_status: &str,
    file_path: &str,
) -> io::Result<()> {
    // Read the file into a vector of lines.
    let file_contents = fs::read_to_string(PathHandler::get_relative_path(INDEX_FILE))?;

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
    fs::write(PathHandler::get_relative_path(INDEX_FILE), updated_contents)?;

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

pub fn get_all_branches() -> Result<Vec<String>, Box<dyn Error>> {
    let current_branch_path = get_current_branch_path()?;
    println!("test: {:?}", current_branch_path);

    // Extract the directory path from the file path
    let dir_path = Path::new(&current_branch_path)
        .parent()
        .ok_or("Failed to get parent directory")?;

    // Remove the ".git/" prefix if it exists
    let dir_path_without_git = dir_path.strip_prefix(".git/").unwrap_or(dir_path);

    // Read the contents of the directory
    let entries = fs::read_dir(dir_path)?;

    // Iterate over the entries
    let mut branches: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry?;

        // Get the file name and content
        let file_name = if entry.path() == Path::new(&current_branch_path) {
            "HEAD".to_string()
        } else {
            dir_path_without_git
                .join(entry.file_name())
                .to_string_lossy()
                .into_owned()
        };
        let file_content = fs::read_to_string(entry.path())?;

        // Combine content and filename
        let branch = format!("{} {}\n", file_content, file_name);
        branches.push(branch);
    }

    Ok(branches)
}

pub fn list_files_recursively(dir_path: &str, files_list: &mut Vec<String>) -> io::Result<()> {
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let entry_path = entry.path();

        if file_type.is_dir() {
            // If it's a subdirectory, recurse into it
            list_files_recursively(
                entry_path
                    .to_str()
                    .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
                files_list,
            )?;
        } else if file_type.is_file() {
            // If it's a file, add its path to the list
            if let Some(current_obj_path) = entry_path.to_str() {
                files_list.push(current_obj_path.to_string());
            }
        }
    }

    Ok(())
}

pub fn get_remote_url(name: &str) -> Result<String, Box<dyn Error>> {
    let config_content = read_file_content(CONFIG_FILE)?;
    let current_remote_line = format!("[remote '{}']", name);
    let mut in_remote = false;

    for line in config_content.lines() {
        if line == current_remote_line.as_str() {
            in_remote = true;
        } else if in_remote {
            let parts: Vec<&str> = line.split(" ").collect();
            let url = parts.last().unwrap_or(&"");
            return Ok(url.to_string());
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "No remote found.",
    )))
}

pub fn generate_sha1_string_from_bytes(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.input(&data);
    hasher.result_str()
}

pub fn read_object(hash: String) -> Result<(ObjectType, String), Box<dyn Error>> {
    
    let file = fs::File::open(PathHandler::get_relative_path(&get_object_path(&hash)))?;
    let mut buffer = String::new();
    Decoder::new(file)?.read_to_string(&mut buffer)?;

    let file_content: Vec<String> = buffer.split('\0').map(String::from).collect();
    let object_header: Vec<String> = file_content[0].split(' ').map(String::from).collect();
    let object_type = ObjectType::new(&object_header[0]).ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Failed to determine object type",
    ))?;

    Ok((object_type, file_content[1].clone()))
}

pub fn get_remote_tracking_branches() -> Result<HashMap<String, (String, String)>, Box<dyn Error>> {
    // Assuming CONFIG_FILE is a constant containing the path to the Git configuration file
    const CONFIG_FILE: &str = ".git/config";

    // Read the content of the Git configuration file
    let config_content = read_file_content(CONFIG_FILE)?;

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

            while let Some(next_line) = lines.next() {
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
                // branches_and_remotes.insert(branch_name.to_string(), format!("{}/{}", remote, merge));
                branches_and_remotes.insert(branch_name.to_string(), (remote, merge));
            }
        }
    }
    Ok(branches_and_remotes)
}

pub fn update_local_branch_with_commit(
    remote_name: &str,
    branch_name: &str,
    remote_hash: &str,
) -> Result<(), Box<dyn Error>> {
    let config_content = read_file_content(CONFIG_FILE)?;

    let branch_header = format!("[branch '{}']", branch_name);
    let mut lines = config_content.lines().peekable();
    while let Some(line) = lines.next() {
        if line == branch_header {
            let mut remote = None;
            while let Some(next_line) = lines.next() {
                if next_line.starts_with("remote = ") {
                    remote = Some(next_line.trim_start_matches("remote = ").to_string());
                } else if next_line.starts_with('[') {
                    break;
                }
            }
            if let Some(remote) = remote {
                if remote == remote_name {
                    let _ = update_branch_hash(branch_name, remote_hash);
                }
            }
        }
    }
    Ok(())
}

fn update_branch_hash(branch_name: &str, new_commit_hash: &str) -> Result<(), Box<dyn Error>> {
    let path = format!("{}/{}", R_HEADS, branch_name);
    let mut file = fs::File::create(path)?;
    file.write_all(new_commit_hash.as_bytes())?;
    Ok(())
}

pub fn get_branch_last_commit(branch_path: &str) -> Result<String, Box<dyn Error>> {
    let mut file: fs::File = fs::File::open(branch_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn get_branch_path(branch_name: &str) -> String {
    format!("{}/{}", R_HEADS, branch_name)
}

pub fn get_object_path(object_hash: &str) -> String {
    format!("{}/{}/{}", OBJECT, object_hash[..2].to_string(), object_hash[2..].to_string())
}

pub fn find_common_ancestor_commit(current_branch: &str, merging_branch: &str) -> Result<String, Box<dyn Error>> {
    let mut current_branch_log = Vec::new();
    let current_branch_commit = get_branch_last_commit(&get_current_branch_path()?)?;
    Log::new().generate_log_entries(&mut current_branch_log, current_branch_commit);
    println!("current branch log: {:?}", current_branch_log);


    let mut merging_branch_log = Vec::new();
    let merging_branch_commit = get_branch_last_commit(&get_branch_path(merging_branch))?;
    Log::new().generate_log_entries(&mut merging_branch_log, merging_branch_commit);
    println!("merging branch log: {:?}", merging_branch_log);
    // tal vez eso parametrizarlo en una funcion

    for (commit, _message) in merging_branch_log {
        if current_branch_log.contains(&(commit.clone(), _message)) {
            return Ok(commit);
        }
    }

    Ok(String::new())
}

pub fn is_fast_forward_merge_possible(current_branch: &str, merging_branch: &str) -> Result<String, Box<dyn Error>> {
    let current_branch_commit = get_branch_last_commit(&get_current_branch_path()?)?;
    println!("current commit: {}", current_branch_commit);
    let mut merging_branch_log = Vec::new();
    let merging_branch_commit = get_branch_last_commit(&get_branch_path(merging_branch))?;
    println!("mergin commit: {}", merging_branch_commit);
    if current_branch_commit.is_empty() {
        println!("trtue");
        return Ok(merging_branch_commit);
    }
    
    Log::new().generate_log_entries(&mut merging_branch_log, merging_branch_commit.clone());

    for (commit, _message) in merging_branch_log {
        if commit == current_branch_commit{
            return Ok(merging_branch_commit);
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "No fast forward is possible",
    )))
}

/// Given a commit's hash it accesses its file and returns the hash of its associated
/// tree object.
pub fn get_commit_tree(commit_hash: &str) -> Result<String, Box<dyn Error>> {
    println!("commit hash: {}", commit_hash);
    let decompressed_data = decompress_file_content(read_file_content_to_bytes(&get_object_path(commit_hash))?)?;

    let commit_file_content: Vec<String> = decompressed_data.split('\0').map(String::from).collect();

    let commit_file_lines: Vec<String> = commit_file_content[1].lines().map(|s| s.to_string()).collect();
    let tree_split_line: Vec<String> = commit_file_lines[0].split_whitespace().map(String::from).collect();
    
    let tree_hash_trimmed = &tree_split_line[1];

    Ok(tree_hash_trimmed.to_string())
}
