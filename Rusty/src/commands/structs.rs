use std::{collections::HashMap, error::Error, fmt, fs, io, io::Write, path::Path, path::PathBuf, env};

use crate::constants::{OBJECT, INDEX_FILE, TREE_SUBTREE_MODE, TREE_FILE_MODE, DEFAULT_HEAD_LINE, HEAD_FILE};

use crate::commands::helpers;
use chrono::{DateTime, Local};
use super::{git_commands::PathHandler, helpers::get_file_length};

/// Struct to interact with the HEAD file in the .git directory.
/// Allows access to information about current branch and last commit in current branch.
pub struct Head;

impl Head {
    /// Changes the branch the HEAD file points to
    pub fn change_head_branch(branch_name: &str) -> Result<(), Box<dyn Error>> {
        let mut head_file = fs::File::create(PathHandler::get_relative_path(HEAD_FILE))?;
        let new_line = format!("{}{}", DEFAULT_HEAD_LINE, branch_name);
        head_file.write_all(new_line.as_bytes())?;
        Ok(())
    }

    /// Returns the ref the HEAD points to
    pub fn get_current_branch_ref() -> Result<String, Box<dyn Error>> {
        println!("ref: {}", &PathHandler::get_relative_path(HEAD_FILE));
        let head_file_content =
            helpers::read_file_content(&PathHandler::get_relative_path(HEAD_FILE))?;
            println!("after reading");
        let split_head_content: Vec<String> = head_file_content
            .split_whitespace()
            .map(String::from)
            .collect();

        let current_ref = split_head_content[1].clone();

        Ok(current_ref)
    }

    /// Returns the name of the current branch
    pub fn get_current_branch_name() -> Result<String, Box<dyn Error>> {
        let current_branch_ref = Self::get_current_branch_ref()?;
        let split_ref_content: Vec<String> =
            current_branch_ref.split('/').map(String::from).collect();

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
        println!("{}", current_branch_path);
        let commit_hash =
            helpers::read_file_content(&PathHandler::get_relative_path(&current_branch_path))?;
        Ok(commit_hash)
    }
}

/// Abstract struct for creating new objects in git repository
pub struct HashObjectCreator;

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
        // println!("data: {:?}", data);
        let hashed_data =
            Self::generate_object_hash(obj_type, content.as_bytes().len() as u64, &content);
        // println!("hash for: {} ; {}", obj_type, hashed_data);
        let compressed_content = helpers::compress_content(&data)?;
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

        let mut object_file = fs::File::create(object_file_path)?;
        object_file.write_all(&compressed_content)?;

        Ok(hashed_data)
    }

    pub fn write_object_file_bytes(
        content: &[u8],
        object_type: ObjectType,
        size: usize,
    ) -> Result<String, Box<dyn Error>> {
        let mut object_content = Vec::new();
        let header = format!("{} {}\0", object_type, size);

        object_content.extend_from_slice(header.as_bytes());
        object_content.extend_from_slice(content);

        let hashed_data = helpers::generate_sha1_string_from_bytes(&object_content);

        // println!("hash: {}", hashed_data);

        let obj_directory_path = format!("{}/{}", OBJECT, &hashed_data[0..2]);
        let _ = fs::create_dir(PathHandler::get_relative_path(&obj_directory_path));

        let object_file_path = format!(
            "{}/{}",
            PathHandler::get_relative_path(&obj_directory_path),
            &hashed_data[2..]
        );
        let compressed_data = helpers::compress_bytes(&object_content)?;
        let mut object_file = fs::File::create(object_file_path)?;
        object_file.write_all(&compressed_data)?;

        Ok(hashed_data)
    }

    pub fn generate_object_hash(obj_type: ObjectType, file_len: u64, content: &str) -> String {
        let data = format!("{} {}\0{}", obj_type, file_len, content);
        // println!("data when generating object hash: {}", data);
        helpers::generate_sha1_string(data.as_str())
    }

    pub fn create_tree_object() -> Result<String, Box<dyn Error>> {
        let index_file_content =
            helpers::read_file_content(&PathHandler::get_relative_path(INDEX_FILE))?;
        let mut subdirectories: HashMap<String, Vec<Vec<u8>>> = HashMap::new();

        let index_file_lines: Vec<&str> = index_file_content.split('\n').collect();
        let mut staged_files = false;

        for line in index_file_lines {
            let split_line: Vec<&str> = line.split(';').collect();
            if split_line[2] == IndexFileEntryState::Cached.to_string() {
                continue;
            }
            staged_files = true;
            let path = Path::new(split_line[0]);
            let hash = split_line[1];

            let mut current_dir = path.parent();
            let mut file_directory = String::new();
            if let Some(directory) = current_dir {
                file_directory = directory.to_string_lossy().to_string();
            }

            let _split_path: Vec<&str> = index_file_content.split('/').collect();
            let mut file_name = String::new();

            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                file_name = name.to_string();
            }
            let hash = helpers::convert_hash_to_decimal_bytes(hash)?;
            let file_entry = format!("{} {}\0", TREE_FILE_MODE, file_name);
            let mut final_entry = Vec::new();
            final_entry.extend_from_slice(file_entry.as_bytes());
            final_entry.extend_from_slice(&hash);

            if let Some(_parent) = current_dir {
                subdirectories
                    .entry(file_directory)
                    .or_default()
                    .push(final_entry);
            }

            while let Some(parent) = current_dir {
                current_dir = parent.parent();
                let subdirectory_entry;
                if let Some(directory) = current_dir {
                    subdirectory_entry = directory.to_string_lossy().to_string();
                    subdirectories
                        .entry(subdirectory_entry)
                        .or_default()
                        .push(parent.to_string_lossy().as_bytes().to_vec());
                }
            }
        }
        let mut super_tree_hash = String::new();
        for (parent_directory, entries) in &subdirectories {
            let sub_tree_content =
                Self::process_files_and_subdirectories(&mut subdirectories.clone(), entries)?;
            let tree_hash = Self::write_object_file_bytes(
                &sub_tree_content,
                ObjectType::Tree,
                sub_tree_content.len(),
            )?;
            if parent_directory == "/" || parent_directory.is_empty() {
                super_tree_hash = tree_hash;
            }
        }
        if !staged_files {
            return Ok(String::new())
        }
        Ok(super_tree_hash)
    }

    fn process_files_and_subdirectories(
        subdirectories: &mut HashMap<String, Vec<Vec<u8>>>,
        entries: &Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut sub_tree_content = Vec::new();
        for entry in entries {
            if !entry.starts_with(TREE_FILE_MODE.as_bytes()) {
                if let Some(value) =
                    subdirectories.remove(&String::from_utf8_lossy(entry).to_string())
                {
                    let mut directory_name = String::new();
                    if let Some(file_name) = String::from_utf8_lossy(entry)
                        .to_string()
                        .rsplit('/')
                        .next()
                    {
                        // println!("File name: {}", file_name);
                        directory_name = file_name.to_string();
                    }
                    let tree_content =
                        Self::process_files_and_subdirectories(subdirectories, &value)?;
                    let tree_hash = Self::write_object_file_bytes(
                        &tree_content,
                        ObjectType::Tree,
                        tree_content.len(),
                    )?;
                    let hash_decimal = helpers::convert_hash_to_decimal_bytes(&tree_hash)?;
                    let tree_entry = format!("{} {}\0", TREE_SUBTREE_MODE, directory_name);
                    let mut final_entry = Vec::new();
                    final_entry.extend_from_slice(tree_entry.as_bytes());
                    final_entry.extend_from_slice(&hash_decimal);
                    sub_tree_content.extend_from_slice(&final_entry);
                }
            } else {
                sub_tree_content.extend_from_slice(entry);
            }
        }
        Ok(sub_tree_content)
    }

    // aca podria hacer una funcion para crear un commit object con dos padres
    // tal vez podria pasar lo de generar contenido del comando para aca, seria
    // mejor division de tareas ahi
    pub fn create_commit_object(message: Option<&str>, parents: Vec<String>) -> Result<String, Box<dyn Error>> {
        let tree_hash = HashObjectCreator::create_tree_object()?;
        if tree_hash.is_empty() {
            // This means there was nothing staged, so no commit should be created.
            println!("no changes added to commit (use 'git add')");
            return Ok(String::new())
        }
        let commit_content = Self::generate_commit_content(tree_hash, message, parents)?;
        println!("commit content: {}", commit_content);
        let commit_object_hash = HashObjectCreator::write_object_file(
            commit_content.clone(),
            ObjectType::Commit,
            commit_content.as_bytes().len() as u64,
        )?;
        
        Ok(commit_object_hash)
    }

    fn generate_commit_content(
        tree_hash: String,
        message: Option<&str>,
        parents: Vec<String>
    ) -> Result<String, Box<dyn Error>> {
        let username = env::var("USER")?;
        let current_time: DateTime<Local> = Local::now();
        let timestamp = current_time.timestamp();

        let offset_minutes = current_time.offset().local_minus_utc();
        let offset_hours = (offset_minutes / 60) / 60;

        let offset_string = format!("{:03}{:02}", offset_hours, (offset_minutes % 60).abs());

        let author_line = format!(
            "author {} <{}@fi.uba.ar> {} {}",
            username, username, timestamp, offset_string
        );
        let commiter_line = format!(
            "committer {} <{}@fi.uba.ar> {} {}",
            username, username, timestamp, offset_string
        );
        let mut content = format!("tree {}\n", tree_hash);
        let mut parents_string = String::new();
        if !parents.is_empty() {
            for parent in parents {
                parents_string = format!("{}parent {}\n", parents_string, parent)
            }
            content = format!("{}{}", content, parents_string);
        }
        content = format!("{}{}\n{}\n", content, author_line, commiter_line);
        if let Some(message) = message {
            content = format!("{}\n{}", content, message);
        }
        Ok(content)
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
        match self {
            IndexFileEntryState::Cached => 0,
            IndexFileEntryState::Modified => 1,
            IndexFileEntryState::Staged => 2,
            IndexFileEntryState::Deleted => 3,
        }
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

impl Default for StagingArea {
    fn default() -> Self {
        Self::new()
    }
}

impl StagingArea {
    pub fn new() -> Self {
        StagingArea {}
    }

    /// Adds a file to the staging area. Creating a git object and saving the object's path, hash and state in the
    /// index file, following the format: file_path;hash;state.
    pub fn add_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let relative_path = PathHandler::get_relative_path(path);
        let file_content = helpers::read_file_content(relative_path.as_str())?;
        let object_hash = HashObjectCreator::write_object_file(
            file_content,
            ObjectType::Blob,
            get_file_length(relative_path.as_str())?,
        )?;
        helpers::update_file_with_hash(
            object_hash.as_str(),
            IndexFileEntryState::Staged.to_string().as_str(),
            path,
        )?;

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
                    lines[index].replace_range(
                        state_index + 1..state_index + 2,
                        IndexFileEntryState::Deleted.to_string().as_str(),
                    );
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
        let index_file_content =
            helpers::read_file_content(&PathHandler::get_relative_path(INDEX_FILE))?;
        let mut lines: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();
        let mut new_index_file_content: Vec<String> = Vec::new();
        
        for line in lines.iter_mut() {
            line.pop();
            line.push_str(IndexFileEntryState::Cached.to_string().as_str());
            new_index_file_content.push(line.to_string());
        }

        let mut index_file = fs::File::create(PathHandler::get_relative_path(INDEX_FILE))?;
        index_file.write_all(new_index_file_content.join("\n").as_bytes())?;
        Ok(())
    }

    pub fn get_entries_index_file(
        &self,
        state: IndexFileEntryState,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let index_file_content = helpers::read_file_content(INDEX_FILE)?;
        let lines: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();

        let mut result: Vec<String> = Vec::new();

        for line in lines {
            let parts: Vec<&str> = line.split(';').map(|s| s.trim()).collect();

            // Assuming each line in the index file has at least two parts: state and file path
            if parts.len() >= 3 {
                let file_name = parts[0];
                let file_state = parts[2];

                if let IndexFileEntryState::Cached = state {
                    result.push(file_name.to_string());
                    continue;
                }

                if file_state == state.to_string() {
                    result.push(file_name.to_string());
                }
            }
        }

        Ok(result)
    }


    pub fn change_index_file(&self, working_tree: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
        let mut new_index_lines: Vec<String> = Vec::new();
        for (file_name, file_hash) in working_tree {
            let new_line = format!("{};{};{}", file_name, file_hash, IndexFileEntryState::Staged);
            new_index_lines.push(new_line);
        }
        let new_index_content = new_index_lines.join("\n");
        let mut index_file = fs::File::create(PathHandler::get_relative_path(INDEX_FILE))?;
        _ = index_file.write_all(new_index_content.as_bytes())?;
        println!("new content:\n{}", new_index_content);
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
        match self {
            ObjectType::Commit => vec![0, 0, 1],
            ObjectType::Tree => vec![0, 1, 0],
            ObjectType::Blob => vec![0, 1, 1],
            ObjectType::Tag => vec![1, 0, 0],
        }
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
        let _ = fs::remove_file(file_path);

        let mut current_dir = file_path.parent();

        while let Some(parent) = current_dir {
            println!("{:?}", parent);
            if parent == Path::new("") {
                break;
            }
            if let Ok(mut dir_entries) = fs::read_dir(parent) {
                println!("entreies: {:?}", dir_entries);
                // println!("{:?}", dir_entries.next());
                let next_dir_entry = dir_entries.next();
                if next_dir_entry.is_none() || next_dir_entry ==  {
                    println!("remove dir");
                    let _ = fs::remove_dir(parent);
                    current_dir = parent.parent();
                    println!("current: {:?}", current_dir);
                }
                
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Goes through the files in the index file and deletes them from the working directory.
    /// These files will still be stored as blob objects, and can be created again if needed.
    pub fn clean_working_directory() -> Result<(), Box<dyn Error>> {
        let index_file_content =
            helpers::read_file_content(&PathHandler::get_relative_path(INDEX_FILE))?;
        let lines: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();

        for line in lines.iter() {
            let split_line: Vec<String> = line.split(';').map(String::from).collect();
            let file_path_str = PathHandler::get_relative_path(&split_line[0].clone());
            println!("path to delete: {}", file_path_str);
            let file_path = PathBuf::from(file_path_str);
            Self::remove_file_and_empty_parent_directories(&file_path)?;
        }

        Ok(())
    }

    /// Creates the files and directories corresponding to the working tree that a tree
    /// object has saved.
    pub fn update_working_directory_to(new_tree: &str) -> Result<(), Box<dyn Error>> {
        let _ = Self::create_files_for_directory(new_tree, &PathHandler::get_relative_path(""));

        Ok(())
    }

    fn create_files_for_directory(
        tree: &str,
        current_directory: &str,
    ) -> Result<(), Box<dyn Error>> {
        let tree_content = helpers::read_tree_content(tree)?;
        for (file_mode, file_name, file_hash) in tree_content {
            let relative_file_path = format!("{}{}", current_directory, file_name);

            match file_mode.as_str() {
                TREE_FILE_MODE => {
                    let (_, object_content, _) = helpers::read_object_to_string(file_hash)?;
                    // println!("object to write content: {} in path: {}", object_content, relative_file_path);
                    let mut object_file = fs::File::create(relative_file_path)?;
                    object_file.write_all(object_content.as_bytes())?;
                }
                TREE_SUBTREE_MODE => {
                    if let Err(_error) = fs::metadata(relative_file_path.clone()) {
                        // println!("creating dir: {}", relative_file_path);
                        fs::create_dir(relative_file_path.clone())?;
                    }
                    let dir_path = format!("{}/", relative_file_path);
                    return Self::create_files_for_directory(&file_hash, &dir_path);
                }
                _ => {}
            }
        }

        Ok(())
    }
}

/* pub const RELATIVE_PATH: &str = "RELATIVE_PATH";
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
 */
