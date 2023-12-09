use std::{
    collections::HashMap, error::Error, fs, io, io::Read, io::Write, path::Path,
};
extern crate crypto;
extern crate libflate;

use crate::commands::{commands::Log, structs::Head};
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
    let current_branch_path = Head::get_current_branch_path()?;
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

pub fn read_object(hash: String) -> Result<(ObjectType, String, String), Box<dyn Error>> {

    let mut file = fs::File::open(PathHandler::get_relative_path(&get_object_path(&hash)))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer);
    let file_data = decompress_file_content(buffer)?;

    let file_content: Vec<String> = file_data.split('\0').map(String::from).collect();
    let object_header: Vec<String> = file_content[0].split(' ').map(String::from).collect();
    let object_type = ObjectType::new(&object_header[0]).ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Failed to determine object type",
    ))?;
    let object_size = if object_header.len() >= 2 { object_header[1].clone() } else { String::new() };
    Ok((object_type, file_content[1].clone(), object_size))
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

pub fn update_branch_hash(branch_name: &str, new_commit_hash: &str) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create(get_branch_path(branch_name))?;
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
    println!("{}", object_hash);
    format!("{}/{}/{}", OBJECT, object_hash[..2].to_string(), object_hash[2..].to_string())
}

pub fn find_common_ancestor_commit(_current_branch: &str, merging_branch: &str) -> Result<String, Box<dyn Error>> {
    let mut current_branch_log = Vec::new();
    let current_branch_commit = Head::get_head_commit()?;
    let _ = Log::new().generate_log_entries(&mut current_branch_log, current_branch_commit);
    println!("current branch log: {:?}", current_branch_log);


    let mut merging_branch_log = Vec::new();
    let merging_branch_commit = get_branch_last_commit(&get_branch_path(merging_branch))?;
    let _ = Log::new().generate_log_entries(&mut merging_branch_log, merging_branch_commit);
    println!("merging branch log: {:?}", merging_branch_log);
    // tal vez eso parametrizarlo en una funcion

    for (commit, _message) in merging_branch_log {
        if current_branch_log.contains(&(commit.clone(), _message)) {
            return Ok(commit);
        }
    }

    Ok(String::new())
}

pub fn ancestor_commit_exists(current_commit_hash: &str, merging_commit_hash: &str) -> Result<bool, Box<dyn Error>> {
    // let current_branch_commit = Head::get_head_commit()?;
    // println!("current commit: {}", current_branch_commit);
    let mut merging_branch_log = Vec::new();
    // aca rompe al hacer con fetch porque estamos queriendo unir una branch que esta en remotes, tal vez ya habria que pasar los hash de commits como parametro
    // de cambiar eso el nombre pasaria a ser tipo ancestor_commit_exists()
    // let merging_branch_commit = get_branch_last_commit(&get_branch_path(merging_branch))?;
    println!("mergin commitg: {}", merging_commit_hash);
    if current_commit_hash.is_empty() {
        println!("true");
        return Ok(true);
    }
    
    println!("generating log...");
    let _ = Log::new().generate_log_entries(&mut merging_branch_log, merging_commit_hash.to_string());
    println!("log: {:?}", merging_branch_log);
    for (commit, _message) in merging_branch_log {
        println!("commit: {} == current commit: {} ", commit, current_commit_hash);
        if commit == current_commit_hash {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Given a commit's hash it accesses its file and returns the hash of its associated
/// tree object.
pub fn get_commit_tree(commit_hash: &str) -> Result<String, Box<dyn Error>> {
    //println!("commit hash: {}", commit_hash);
    let decompressed_data = decompress_file_content(read_file_content_to_bytes(&PathHandler::get_relative_path(&get_object_path(commit_hash)))?)?;

    let commit_file_content: Vec<String> = decompressed_data.split('\0').map(String::from).collect();
    //println!("commit_file_content: {:?}", commit_file_content);

    let commit_file_lines: Vec<String> = commit_file_content[1].lines().map(|s| s.to_string()).collect();
    //println!("commit_file_lines: {:?}", commit_file_lines);
    let tree_split_line: Vec<String> = commit_file_lines[0].split_whitespace().map(String::from).collect();
    //println!("tree_split_line: {:?}", tree_split_line);
    
    let tree_hash_trimmed = &tree_split_line[1];

    Ok(tree_hash_trimmed.to_string())
}

/// Checks if the file in the given path exists and returns true or false
pub fn check_if_file_exists(file_path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(PathHandler::get_relative_path(file_path)) {
        if metadata.is_file() {
            return true
        }
    }
    false
}

/// Checks if the directory in the given path exists and returns true or false
pub fn check_if_directory_exists(dir_path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(dir_path) {
        if metadata.is_dir() {
            return true
        }
    }
    false
}

fn hex_string_to_bytes(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    let mut hash: String = String::new();
    for byte in bytes {
        println!("{:x}", byte);
        hash.push_str(&format!("{:x}", byte));
    }

    Ok(hash)
}

pub fn read_tree_content(tree_hash: &str) -> Result<Vec<(String, String, String)>, Box<dyn Error>> {
    //  [230, 157, 226, 155, 178, 209, 214, 67, 75, 139, 41, 174, 119, 90, 216, 194, 228, 140, 83, 145]
    println!("reading tree content..");
    // let tree_content = decompress_file_content(read_file_content_to_bytes(&get_object_path(tree_hash))?)?;
    // println!("tree content: {}", tree_content);
    // let split_content: Vec<String> = tree_content.splitn(2, '\0').map(String::from).collect();

    let mut file = fs::File::open(get_object_path(tree_hash))?;
    let mut buffer = Vec::new();
    println!("before reading file");
    file.read_to_end(&mut buffer);
    // Decoder::new(file)?.read_to_end(&mut buffer)?;
    println!("compressed data: {:?}", buffer);
    let decompressed = decompress_file_content(buffer)?;
    println!("tree content: {}", decompressed);
    // let buffer_to_string = String::from_utf8_lossy(&decompressed).to_string();
    let split_content: Vec<String> = decompressed.splitn(2, '\0').map(String::from).collect();

    let mut divided_content = Vec::new();
    println!("tree split_content: {}", split_content[1]);
    let mut substrings: Vec<String> = split_content[1].split("\0").map(String::from).collect();
    println!("substrings: {:?}", substrings);
    // Process each substring of 20 bytes
    let tree_data: Vec<String> = substrings[0].split_whitespace().map(String::from).collect();
    let mut file_mode = tree_data[0].clone();
    let mut file_name = tree_data[1].clone();
    substrings.remove(0);
    for substring in substrings {
        let substring_bytes = substring.bytes();
        println!("substring bytes: {:?}", substring_bytes);
        // let processed_bytes = &substring_bytes[..20];
        let processed_bytes: Vec<u8> = substring_bytes.take(20).collect();
        println!("bytes: {:?}", processed_bytes);
        let hash_string = hex_string_to_bytes(&processed_bytes)?;
        // Perform your processing here, for example, print the processed bytes
        println!("final content: {} {} {}", file_mode.clone(), file_name.clone(), hash_string);
        divided_content.push((file_mode.clone(), file_name.clone(), hash_string));
        if substring.len() > 20 {
            let tree_entry_data = &substring[substring.len()..];
            println!("entry data: {}", tree_entry_data);
            let split_entry: Vec<String> = tree_entry_data.split_whitespace().map(String::from).collect();
            file_mode = split_entry[0].clone();
            file_name = split_entry[1].clone();
        }
    }
    println!("{:?}", divided_content);
    Ok(divided_content)
}

pub fn convert_hash_to_decimal_bytes(hash: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut decimal_hash = Vec::new();
    for chunk in hash.chars().collect::<Vec<char>>().chunks(2) {
        let chunk_str: String = chunk.iter().collect();
        let result = u8::from_str_radix(&chunk_str, 16)?;
        decimal_hash.push(result);
    }

    println!("hash: {:?}", decimal_hash);
    Ok(decimal_hash)
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