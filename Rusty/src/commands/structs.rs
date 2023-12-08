use std::{error::Error, fmt, fs, io::BufRead, io::Write, net::TcpStream, io, path::PathBuf, path::Path, collections::HashMap};
const OBJECT: &str = ".git/objects";
const INDEX_FILE: &str = ".git/index";
const TREE_SUBTREE_MODE: &str = "040000";
const TREE_FILE_MODE: &str = "100644";
const DEFAULT_HEAD_LINE: &str = "ref: refs/heads/";
const HEAD_FILE: &str = ".git/HEAD";
//const DEFAULT_REMOTE: &str = "origin";

use crate::commands::helpers;

use super::{commands::PathHandler, helpers::get_file_length};

/// Struct to interact with the HEAD file in the .git directory.
/// Allows access to information about current branch and last commit in current branch.
pub struct Head;

impl Head {
    /// Changes the branch the HEAD file points to
    pub fn change_head_branch(branch_name: &str) -> Result<(), Box<dyn Error>>{
        let mut head_file = fs::File::create(&PathHandler::get_relative_path(HEAD_FILE))?;
        let new_line = format!("{}{}", DEFAULT_HEAD_LINE, branch_name);
        head_file.write_all(new_line.as_bytes())?;
        Ok(())
    }

    /// Returns the ref the HEAD points to
    pub fn get_current_branch_ref() -> Result<String, Box<dyn Error>> {
        let head_file_content = helpers::read_file_content(&PathHandler::get_relative_path(HEAD_FILE))?;
        let split_head_content: Vec<String> = head_file_content.split_whitespace().map(String::from).collect();

        let current_ref = split_head_content[1].clone();

        Ok(current_ref)
    }

    /// Returns the name of the current branch
    pub fn get_current_branch_name() -> Result<String, Box<dyn Error>> {
        let current_branch_ref = Self::get_current_branch_ref()?;
        let split_ref_content: Vec<String> = current_branch_ref.split('/').map(String::from).collect();

        let branch_name = split_ref_content[2].clone();

        Ok(branch_name)
    }

    /// Returns the path of the current branch
    pub fn get_current_branch_path() -> Result<String, Box<dyn Error>> {
        let current_branch_name = Self::get_current_branch_name()?;
        Ok(helpers::get_branch_path(&current_branch_name))
    }

    /// Returns the last commit of the current branch
    pub fn get_head_commit() -> Result<String, Box<dyn Error>> {
        let current_branch_path = Self::get_current_branch_path()?;
        let commit_hash = helpers::read_file_content(&PathHandler::get_relative_path(&current_branch_path))?;
        Ok(commit_hash)
    }
}

/// Abstract struct for creating new objects in git repository
pub struct HashObjectCreator;

fn hex_string_to_bytes(hex_string: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    let mut chars = hex_string.chars();

    while let Some(a) = chars.next() {
        if let Some(b) = chars.next() {
            let byte = u8::from_str_radix(&format!("{}{}", a, b), 16)?;
            bytes.push(byte);
        } else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Invalid Commit ID. It's too short",
            )));
        }
    }

    Ok(bytes)
}
impl HashObjectCreator {
    /// Writes an object file to the Git repository.
    ///
    /// This function takes the provided content, object type, and file length, and writes the object
    /// data to a file in the Git repository. The content is first formatted with object type and file
    /// length, hashed, and then compressed before being written to the repository.
    /// Returns a Result that may contain a string of the hash of the written object.
    pub fn write_object_file(
        content: String,
        obj_type: ObjectType,
        file_len: u64,
    ) -> Result<String, Box<dyn Error>> {
        let data = format!("{} {}\0{}", obj_type, file_len, content);
        println!("data: {:?}", data);
        // if obj_type == ObjectType::Tree {
        //     println!("last part of data: {}", &data[29..]);
        //     let bytes = hex_string_to_bytes(&data[29..])?;

        //     // Convert bytes to a string
        //     let readable_hash = String::from_utf8_lossy(&bytes);
        //     println!("readable hash: {}", readable_hash.to_string());
        // }
        let hashed_data = Self::generate_object_hash(obj_type.clone(), file_len, &content);
        println!("hash for: {} ; {}", obj_type, hashed_data);
        let compressed_content = helpers::compress_content(data.as_str())?;
        let obj_directory_path = format!("{}/{}", OBJECT, &hashed_data[0..2]);
        let _ = fs::create_dir(PathHandler::get_relative_path(&obj_directory_path));

        let object_file_path = format!(
            "{}/{}",
            PathHandler::get_relative_path(&obj_directory_path),
            &hashed_data[2..]
        );
        if fs::metadata(object_file_path.clone()).is_ok() {
            return Ok(hashed_data);
        }

        let mut object_file = fs::File::create(&object_file_path.clone())?;
        object_file.write_all(&compressed_content)?;

        Ok(hashed_data)
    }

    pub fn write_object_file_bytes(
        content: Vec<u8>,
        obj_type: ObjectType,
        file_len: u64,
    ) -> Result<String, Box<dyn Error>> {
        let header = format!("{} {}\0", obj_type, file_len);
        println!("header: {}", header);
        let mut final_content: Vec<u8> = Vec::new();
        final_content.extend_from_slice(header.as_bytes());
        final_content.extend_from_slice(&content);
        let hashed_data = helpers::generate_sha1_string_from_bytes(&content);
        println!("hash for: {} ; {}", obj_type, hashed_data);
        // let compressed_content = helpers::compress_content(data.as_str())?;
        // let obj_directory_path = format!("{}/{}", OBJECT, &hashed_data[0..2]);
        // let _ = fs::create_dir(PathHandler::get_relative_path(&obj_directory_path));

        // let object_file_path = format!(
        //     "{}/{}",
        //     PathHandler::get_relative_path(&obj_directory_path),
        //     &hashed_data[2..]
        // );
        // if fs::metadata(object_file_path.clone()).is_ok() {
        //     return Ok(hashed_data);
        // }

        // let mut object_file = fs::File::create(&object_file_path.clone())?;
        // object_file.write_all(&compressed_content)?;

        Ok(hashed_data)
    }

    pub fn generate_object_hash(obj_type: ObjectType, file_len: u64, content: &str) -> String {
        let data = format!("{} {}\0{}", obj_type, file_len, content);
        helpers::generate_sha1_string(data.as_str())
    }

    pub fn create_tree_object() -> Result<String, Box<dyn Error>> {
        let index_file_content = helpers::read_file_content(&PathHandler::get_relative_path(INDEX_FILE))?;
        let mut subdirectories: HashMap<String, Vec<String>> = HashMap::new();
    
        let index_file_lines: Vec<&str> = index_file_content.split("\n").collect();
        //println!("index_file_lines: {:?}", index_file_lines);
    
        for line in index_file_lines {
            let split_line: Vec<&str> = line.split(";").collect();

            let path = Path::new(split_line[0]);
            let hash = split_line[1];
            
            let mut current_dir = path.parent();
            let mut file_directory = String::new();
            //println!("current_dir: {:?}", current_dir);
            if let Some(directory) = current_dir {
                file_directory = directory.to_string_lossy().to_string();
            }

            let _split_path: Vec<&str> = index_file_content.split("/").collect();
            let mut file_name = String::new();

            //println!("_split_path: {:?}", _split_path);
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                //println!("File name: {}", name);
                file_name = name.to_string();
            }
            let file_entry = format!("{} {} {} {}\n", TREE_FILE_MODE, ObjectType::Blob, hash, file_name);

            //println!("file_entry: {:?}", file_entry);
            if let Some(_parent) = current_dir {
                subdirectories
                            .entry(file_directory)
                            .or_insert_with(Vec::new)
                            .push(file_entry);
            }

            while let Some(parent) = current_dir {
                current_dir = parent.parent();
                //println!("current_dir adentro del while: {:?}", current_dir);
                let mut subdirectory_entry = String::new();
                if let Some(directory) = current_dir {
                	//println!("current_dir adentro del while , adentro del iflet: {:?}", directory);
                    subdirectory_entry = directory.to_string_lossy().to_string();
	                subdirectories
	                        .entry(subdirectory_entry)
	                        .or_insert_with(Vec::new)
	                        .push(parent.to_string_lossy().to_string());
                }
            }
        }
        let mut super_tree_hash = String::new();
        for (parent_directory, entries) in &subdirectories {
        	//println!("[create_tree_object]parent_directory: {:?}", parent_directory);
            let sub_tree_content = Self::process_files_and_subdirectories(&mut subdirectories.clone(), &entries)?;
            let tree_hash = Self::write_object_file(sub_tree_content.clone(), ObjectType::Tree, sub_tree_content.len() as u64)?;
            if parent_directory == "/" || parent_directory.is_empty() {
            	//println!("[create_tree_object]inside if: {:?}", parent_directory);
                super_tree_hash = tree_hash;
            }
        }
        Ok(super_tree_hash)
    }

    fn process_files_and_subdirectories(subdirectories: &mut HashMap<String, Vec<String>>, entries: &Vec<String>) -> Result<String, Box<dyn Error>> {
        let mut sub_tree_content = String::new();
        for entry in entries {
            if !entry.starts_with(TREE_FILE_MODE) {
                match subdirectories.remove(&entry.clone()) {
                    Some(value) => {
                        let mut directory_name = String::new();
                        if let Some(file_name) = entry.rsplit('/').next() {
                            println!("File name: {}", file_name);
                            directory_name = file_name.to_string();
                        }
                        let tree_content = Self::process_files_and_subdirectories(subdirectories, &value)?;
                        let tree_hash = Self::write_object_file(tree_content.clone(), ObjectType::Tree, tree_content.len() as u64)?;
                        let tree_entry = format!("{} {} {} {}\n", TREE_SUBTREE_MODE, ObjectType::Tree, tree_hash, directory_name);
                        sub_tree_content.push_str(&tree_entry);
                    }
                    None => {}
                }
            } else {
                sub_tree_content.push_str(entry);
            }
        }
        Ok(sub_tree_content)
    }
}

pub enum IndexFileEntryState {
    Cached,
    Staged,
    Modified,
    Deleted,
}

impl IndexFileEntryState {
    pub fn new(state: &str) -> Option<Self> {
        match state {
            "0" => Some(IndexFileEntryState::Cached),
            "1" => Some(IndexFileEntryState::Modified),
            "2" => Some(IndexFileEntryState::Staged),
            "3" => Some(IndexFileEntryState::Deleted),
            _ => None,
        }
    }
    pub fn get_entry_state_for_file(&self) -> u8 {
        let state = match self {
            IndexFileEntryState::Cached => 0,
            IndexFileEntryState::Modified => 1,
            IndexFileEntryState::Staged => 2,
            IndexFileEntryState::Deleted => 3,
        };
        state
    }
}

impl fmt::Display for IndexFileEntryState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            IndexFileEntryState::Cached => "0",
            IndexFileEntryState::Modified => "1",
            IndexFileEntryState::Staged => "2",
            IndexFileEntryState::Deleted => "3",
        };
        write!(f, "{}", string)
    }
}

/// Represents the staging area for Git. Where files can be added and removed. They can have 3 possible states,
/// stage, modified and untracked.
#[derive(Debug)]
pub struct StagingArea;

impl StagingArea {
    pub fn new() -> Self {
        StagingArea {}
    }

    /// Adds a file to the staging area. Creating a git object and saving the object's path, hash and state in the
    /// index file, following the format: file_path;hash;state.
    pub fn add_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let file_content = helpers::read_file_content(path)?;
        let object_hash = HashObjectCreator::write_object_file(
            file_content,
            ObjectType::Blob,
            get_file_length(path)?,
        )?;
        helpers::update_file_with_hash(&object_hash.as_str(), IndexFileEntryState::Staged.to_string().as_str(), path)?;

        Ok(())
    }

    /// Removes a file from the staging area.
    pub fn remove_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
    	// Read the file into a vector of lines.
	    let file_contents = fs::read_to_string(PathHandler::get_relative_path(INDEX_FILE))?;

	    // Split the file contents into lines.
	    let mut lines: Vec<String> = file_contents.lines().map(|s| s.to_string()).collect();

	    // Search for the hash in the lines.
	    if let Some(index) = lines.iter_mut().position(|line| line.starts_with(path)) {
	        if let Some(state_index) = lines[index].rfind(';') {
	            // Check if there is a state digit after the last ";"
	            if state_index + 1 < lines[index].len() {
	                // Modify the state to "3".
	                lines[index].replace_range(state_index + 1..state_index + 2, IndexFileEntryState::Deleted.to_string().as_str());
	            }
	        }
	    } else {
	        return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Branch name did not match any file known.",
                    )));
	    }

	    // Join the lines back into a single string.
	    let updated_contents = lines.join("\n");

	    // Write the updated contents back to the file.
	    fs::write(PathHandler::get_relative_path(INDEX_FILE), updated_contents)?;

	    Ok(())
    }

    pub fn unstage_index_file(&self) -> Result<(), Box<dyn Error>> {
        let index_file_content = helpers::read_file_content(&PathHandler::get_relative_path(INDEX_FILE))?;
        let mut lines: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();
        let mut new_index_file_content = String::new();

        for line in lines.iter_mut() {
            line.pop();
            line.push_str(IndexFileEntryState::Cached.to_string().as_str());
            new_index_file_content.push_str(line);
            new_index_file_content.push('\n'); // Add a newline between lines
        }

        let mut index_file = fs::File::create(PathHandler::get_relative_path(INDEX_FILE))?;
        index_file.write_all(new_index_file_content.as_bytes())?;
        Ok(())
    }

	pub fn get_entries_index_file(&self, state: IndexFileEntryState) -> Result<Vec<String>, Box<dyn Error>> {
	    let index_file_content = helpers::read_file_content(INDEX_FILE)?;
	    let lines: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();

	    let mut result: Vec<String> = Vec::new();

	    for line in lines {
	        let parts: Vec<&str> = line.split(';').map(|s| s.trim()).collect();

	        // Assuming each line in the index file has at least two parts: state and file path
	        if parts.len() >= 3 {
	            let file_name = parts[0];
	            let file_state = parts[2];

	            match state {
	            	IndexFileEntryState::Cached => {
	            		result.push(file_name.to_string());
	            		continue;
	            	},
	            	_ => {}
	            }

	            if file_state == state.to_string() {	            	
	            	result.push(file_name.to_string());
	            }
	        }
	    }
	    println!("state: {:?}, result: {:?}", state.to_string(), result);

	    Ok(result)
	}
    
    pub fn change_index_file(&self, mut commit_tree: String) -> Result<(), Box<dyn Error>> {
        println!("change index file for tree: {}", commit_tree);
        let mut index_file = fs::File::create(PathHandler::get_relative_path(INDEX_FILE))?;
        let mut new_index_content = String::new();

        println!("antes del tree content");

        // el pull no lee bien el objeto. el tree se esta guardando con un hash distinto
        let (_, tree_content,  _) = helpers::read_object(commit_tree)?;
        println!("tree content");
        
        let tree_lines: Vec<String> = tree_content.lines().map(|s| s.to_string()).collect();
        println!("tree lines: {:?}", tree_lines);
        // let tree_split_line: Vec<String> = commit_file_lines[0].split_whitespace().map(String::from).collect();
        
        // let tree_hash_trimmed = &tree_split_line[1];
        self.create_index_content(&mut new_index_content, &tree_lines, &String::new())?;
        println!("new content: {}", new_index_content);
        index_file.write_all(&new_index_content.as_bytes());
        Ok(())
    }

    fn create_index_content(&self, index_content: &mut String, tree_lines: &Vec<String>, partial_path: &str)  -> Result<(), Box<dyn Error>> {
        for line in tree_lines {
            println!("line: {}", line);
            let split_line: Vec<String> = line.split_whitespace().map(String::from).collect();
            let file_mode = split_line[0].as_str();
            let file_hash = split_line[2].as_str();
            let file_name = split_line[3].as_str();
            if file_mode == TREE_SUBTREE_MODE {
                let directory_path = format!("{}{}/", partial_path, file_name);
                println!("dir path: {}", directory_path);
                let (_, sub_tree_content, _) = helpers::read_object(file_hash.to_string())?;
                let sub_tree_lines: Vec<String> = sub_tree_content.lines().map(|s| s.to_string()).collect();
                self.create_index_content(index_content, &sub_tree_lines, &directory_path)?
            } else {
                let index_line = format!("{}{};{};{}\n", partial_path, file_name, file_hash, IndexFileEntryState::Cached);
                println!("index line: {}", index_line);
                index_content.push_str(index_line.as_str());
            }
        }  
        Ok(())
    }
}

/// Represents the type of a Git object.
#[derive(Clone, PartialEq)]
pub enum ObjectType {
    Blob,
    Commit,
    Tree,
    Tag,
}

impl ObjectType {
    pub fn new(obj_type: &str) -> Option<Self> {
        match obj_type {
            "blob" => Some(ObjectType::Blob),
            "commit" => Some(ObjectType::Commit),
            "tree" => Some(ObjectType::Tree),
            "tag" => Some(ObjectType::Tag),
            _ => None,
        }
    }
    pub fn get_object_for_pack_file(&self) -> Vec<u8> {
        let object_type = match self {
            ObjectType::Commit => vec![0,0,1],
            ObjectType::Tree => vec![0,1,0],
            ObjectType::Blob => vec![0,1,1],
            ObjectType::Tag => vec![1,0,0],
        };
        object_type
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            ObjectType::Blob => "blob",
            ObjectType::Commit => "commit",
            ObjectType::Tree => "tree",
            ObjectType::Tag => "tag",
        };
        write!(f, "{}", string)
    }
}

pub enum PackObjectType {
    Base(ObjectType),
    OffsetDelta,
    HashDelta,
}

pub struct WorkingDirectory;

impl WorkingDirectory {
    fn remove_file_and_empty_parent_directories(file_path: &Path) -> Result<(), Box<dyn Error>> {
        fs::remove_file(file_path)?;
    
        let mut current_dir = file_path.parent();
    
        while let Some(parent) = current_dir {
            println!("{:?}", parent);
            if parent == Path::new("") {
                break;
            }
            if fs::read_dir(parent)?.next().is_none() {
                fs::remove_dir(parent)?;
                current_dir = parent.parent();
            } else {
                break;
            }
        }
        Ok(())
    }
    
    pub fn clean_working_directory() -> Result<(), Box<dyn Error>> {
        println!("cleaning working directory");
        let index_file_content = helpers::read_file_content(&PathHandler::get_relative_path(INDEX_FILE))?;
        let lines: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();
    
        for line in lines.iter() {
            let split_line: Vec<String> = line.split(';').map(String::from).collect();
            let file_path_str = split_line[0].clone();
            println!("path to delete: {}", file_path_str);
            let file_path = PathBuf::from(file_path_str);
            Self::remove_file_and_empty_parent_directories(&file_path)?;
        }
    
        Ok(())
    }
    
    pub fn update_working_directory_to(new_tree: &str) -> Result<(), Box<dyn Error>> {
        println!("new_tree: {}", new_tree);
    
        let _ = Self::create_files_for_directory(new_tree, &String::new());
    
        Ok(())
    }
    
    fn create_files_for_directory(tree: &str, current_directory: &str) -> Result<(), Box<dyn Error>> {
        let (_, tree_content, _) = helpers::read_object(tree.to_string())?;
        let tree_content_lines: Vec<String> = tree_content.lines().map(|s| s.to_string()).collect();
        println!("tree_content: {}", tree_content);
        for line in tree_content_lines {
            println!("line to create: {}", line);
            let split_line: Vec<String> = line.split_whitespace().map(String::from).collect();
            let file_mode = split_line[0].as_str();
            let object_hash = split_line[2].clone();
            let file_path = &split_line[3];
            let relative_file_path = format!("{}{}", current_directory, file_path);
    
            match file_mode {
                TREE_FILE_MODE => {
                    let (_, object_content, _) = helpers::read_object(object_hash)?;
                    println!("object to write content: {} in path: {}", object_content, relative_file_path);
                    let mut object_file = fs::File::create(relative_file_path)?;
                    object_file.write_all(&object_content.as_bytes())?;
                }
                TREE_SUBTREE_MODE => {
                    if let Err(error) = fs::metadata(relative_file_path.clone()) {
                        println!("creating dir: {}", relative_file_path);
                        fs::create_dir(relative_file_path.clone())?;
                    }
                    let dir_path = format!("{}/", relative_file_path);
                    return Self::create_files_for_directory(&object_hash, &dir_path);
                }
                _ => {}
            }
        }
    
        Ok(())
    }
}

pub const RELATIVE_PATH: &str = "RELATIVE_PATH";
#[cfg(test)]
mod tests {
    use std::{env, fs::File};

    use crate::commands::commands::{Init, Command, Add};

    use super::*;

    const TEST_FILE_CONTENT: &str = "Test file content";
    use tempfile::{tempdir, TempDir};

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
    fn test_hash_object_creator() {
        // Common setup
        let _temp_dir = common_setup();
        let _head = Head::new();
        // Test writing object file
        let obj_type = ObjectType::Blob;
        let file_len = TEST_FILE_CONTENT.len() as u64;
        let result = HashObjectCreator::write_object_file(
            TEST_FILE_CONTENT.to_string(),
            obj_type.clone(),
            file_len,
        );

        // Assert that the command executed successfully
        assert!(result.is_ok(), "Write object file failed: {:?}", result);
        let object_hash = result.unwrap();

        // Test generate object hash
        let generated_hash =
            HashObjectCreator::generate_object_hash(obj_type, file_len, TEST_FILE_CONTENT);

        // Assert that the generated hash matches the one obtained from writing the object file
        assert_eq!(generated_hash, object_hash);
    }

    #[test]
    fn test_staging_area() {
        // Common setup
        let _temp_dir = common_setup();

        // Create a new StagingArea instance
        let staging_area = StagingArea::new();

        // Add a file to the staging area
        let file_path = PathHandler::get_relative_path("test.txt");
        fs::write(&file_path, TEST_FILE_CONTENT).expect("Failed to create test file");

        let mut head = Head::new();
        let result = staging_area.add_file(&file_path);

        // Assert that the command executed successfully
        assert!(result.is_ok(), "Add file to staging area failed: {:?}", result);

        // Get staged files from the index file
        let staged_files =
            staging_area.get_entries_index_file(IndexFileEntryState::Staged).unwrap();

        // Assert that the added file is in the list of staged files
        assert!(staged_files.contains(&file_path));
    }

    #[test]
    fn test_clean_working_directory() {
        let (temp_dir, _temp_path) = common_setup();
        // Create a temporary directory and some files
        let mut head = Head::new() ; 
        let file_paths = create_temp_files(&temp_dir, &["file1.txt", "file2.txt"]);

        // Modify the index file to include these files
        let add = Add::new();
        let args1: Option<Vec<&str>> = Some(vec![&file_paths[0]]);
        let args: Option<Vec<&str>> = Some(vec![&file_paths[1]]);
        let _ = add.execute(args1);
        let _ = add.execute( args);

        // Perform clean_working_directory
        WorkingDirectory::clean_working_directory().expect("Failed to clean working directory");

        // Check if the files are deleted
        assert!(!file_exists(&file_paths[0]));
        assert!(!file_exists(&file_paths[1]));

        // Check if the parent directory is deleted
        assert!(!dir_exists(temp_dir.path()));
    }

    #[test]
    fn test_update_working_directory_to() {
        // Create a temporary directory
        let (temp_dir, _temp_path) = common_setup();

        let mut head = Head::new() ; 
        let file_paths = create_temp_files(&temp_dir, &["file1.txt", "file2.txt"]);

        // Modify the index file to include these files
        let add = Add::new();
        let args1: Option<Vec<&str>> = Some(vec![&file_paths[0]]);
        let args: Option<Vec<&str>> = Some(vec![&file_paths[1]]);
        let _ = add.execute(args1);
        let _ = add.execute(args);

        let tree_hash = HashObjectCreator::create_tree_object().unwrap();

        let _ = fs::remove_file(file_paths[0].clone());
        let _ = fs::remove_file(file_paths[1].clone());

        // Perform update_working_directory_to
        WorkingDirectory::update_working_directory_to(&tree_hash).expect("Failed to update working directory");

        // Check if the files are created
        let expected_file_path = temp_dir.path().join("file1.txt");
        assert!(file_exists(&expected_file_path.to_str().unwrap()));

        // Check if the parent directory is created
        assert!(dir_exists(temp_dir.path()));
    }

    // Helper function to create temporary files and return their paths
    fn create_temp_files(temp_dir: &TempDir, file_names: &[&str]) -> Vec<String> {
        let mut file_paths = Vec::new();
        for file_name in file_names {
            let file_path = temp_dir.path().join(file_name);
            write_to_file(&file_path, "Sample file content");
            file_paths.push(file_path.to_str().unwrap().to_string());
        }
        file_paths
    }

    // Helper function to write content to a file
    fn write_to_file(file_path: &std::path::Path, content: &str) {
        let mut file = File::create(file_path).expect("Failed to create file");
        file.write_all(content.as_bytes()).expect("Failed to write to file");
    }

    // Helper function to check if a file exists
    fn file_exists(file_path: &str) -> bool {
        Path::new(file_path).exists()
    }

    // Helper function to check if a directory exists
    fn dir_exists(dir_path: &Path) -> bool {
        dir_path.exists() && dir_path.is_dir()
    }
}
