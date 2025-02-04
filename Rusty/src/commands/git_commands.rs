use std::fmt::Write as Write_FMT;
use std::{
    collections::HashSet, error::Error, fs, io, io::BufRead, io::ErrorKind, io::Read,
    io::Seek, io::SeekFrom, io::Write, str, fs::ReadDir,
};

extern crate libflate;
use libflate::zlib::Decoder;
use crypto::{digest::Digest, sha1::Sha1};

use crate::client;
use crate::client::client_protocol::ClientProtocol;
use crate::commands::helpers::get_file_length;
use crate::commands::helpers;

use crate::commands::structs::*;
use crate::constants::*;
// TODO MOVER A OTRA CARPETA. NO TIENE SENTIDO commands::commands::PathHandler
#[derive(Clone)]
pub struct PathHandler {
    path: String,
}

impl PathHandler {
    pub fn new(path: String) -> Self {
        let path = if !path.ends_with('/') {
            // If the path doesn't end with '/', append it
            format!("{}/", path)
        } else {
            // If the path already ends with '/', leave it unchanged
            path
        };
        PathHandler { path }
    }

    pub fn get_relative_path(&self, append_path: &str) -> String {

        let path = if !self.path.is_empty() {
            // Concatenate with a const string
            format!("{}{}", self.path, append_path)
        } else {
            append_path.to_string()
        };

        if let Some(stripped) = path.strip_prefix('/') {
            // If the input starts with '/', return a new &str without it
            stripped.to_string()
        } else {
            // Otherwise, return the original input as is
            path
        }
    }

    pub fn set_relative_path(&mut self, path: String) {
        let path = if !path.ends_with('/') {
            // If the path doesn't end with '/', append it
            format!("{}/", path)
        } else {
            // If the path already ends with '/', leave it unchanged
            path
        };
        self.path = path;
    }
}

pub trait Command {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>>;
}

pub struct Init;

impl Default for Init {
    fn default() -> Self {
        Self::new()
    }
}

impl Init {
    pub fn new() -> Self {
        Init {}
    }
}

impl Command for Init {
    /// Executes the `git init` command, initializing a new Git repository in the current directory.
    /// This function initializes a new Git repository by creating the necessary directory structure
    /// for branches, tags, and objects. It also sets the default branch to 'main' and creates an empty
    ///  index file. If successful, it returns an empty string; otherwise, it returns an error message.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let arg_slice = args.unwrap_or_default();

        let mut path_handler = path_handler.clone();

        if let Some(&repo_name) = arg_slice.first() {
            if repo_name.contains('/') {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Repository name can't contain the '/' special symbols",
                )));
            }

            // Concatenate a '/' character and call the methods
            path_handler.set_relative_path(path_handler.get_relative_path(repo_name));
        }
        if helpers::check_if_directory_exists(&path_handler.get_relative_path(GIT)) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "A git repository already exists in this directory",
            )));
        }
        let _refs_heads = fs::create_dir_all(path_handler.get_relative_path(R_HEADS));
        fs::create_dir_all(path_handler.get_relative_path(R_TAGS))?;
        fs::create_dir(path_handler.get_relative_path(OBJECT))?;
        fs::create_dir(path_handler.get_relative_path(PACK))?;
        fs::create_dir(path_handler.get_relative_path(R_REMOTES))?;

        let mut _config_file = fs::File::create(path_handler.get_relative_path(CONFIG_FILE))?;
        Branch::new().create_new_branch(DEFAULT_BRANCH_NAME, &path_handler)?;
        Head::change_head_branch(DEFAULT_BRANCH_NAME, &path_handler)?;

        let _index_file = fs::File::create(path_handler.get_relative_path(INDEX_FILE))?;

        Ok(String::new())
    }
}

pub struct Branch;

impl Default for Branch {
    fn default() -> Self {
        Self::new()
    }
}

impl Branch {
    pub fn new() -> Self {
        Branch {}
    }

    /// Creates a new branch with the specified name. Creates branch file.
    pub fn create_new_branch(&self, branch_name: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        let branch_path = helpers::get_branch_path(branch_name);

        if helpers::check_if_file_exists(&branch_path, path_handler) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "A branch with the specified name already exists",
            )));
        }

        let mut branch_file = fs::File::create(path_handler.get_relative_path(&branch_path))?;

        if branch_name == DEFAULT_BRANCH_NAME {
            write!(branch_file, "")?;
        } else {
            let last_commit_hash = Head::get_head_commit(path_handler)?;
            write!(branch_file, "{}", last_commit_hash)?;
        }

        Ok(())
    }

    pub fn delete_branch(&self, branch_name: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        let branch_path = helpers::get_branch_path(branch_name);

        if !helpers::check_if_file_exists(&branch_path, path_handler) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "No branch with the specified name was found",
            )));
        }

        if Head::get_current_branch_name(path_handler)? == branch_name {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Cannot delete current branch",
            )));
        }

        fs::remove_file(path_handler.get_relative_path(&branch_path))?;

        Ok(())
    }

    pub fn list_all_branches(&self, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let mut branches: Vec<String> = Vec::new();

        match fs::read_dir(path_handler.get_relative_path(R_HEADS)) {
            Ok(entries) => {
                for entry in entries {
                    branches.push(entry?.file_name().to_string_lossy().to_string())
                }
            }
            Err(_) => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error reading branches",
                )))
            }
        }

        let current_branch = Head::get_current_branch_name(path_handler)?;

        for branch in branches.clone() {
            if branch == current_branch {
                print!("{}* {}\n{}", COLOR_GREEN_CODE, branch, COLOR_RESET_CODE);
            } else {
                println!("{}", branch)
            }
        }

        let branches_in_string: String =
            branches.into_iter().fold(String::new(), |mut acc, branch| {
                writeln!(acc, "{}\n", branch).expect("Error writing to String");
                acc
            });

        Ok(branches_in_string)
    }

    pub fn rename_branch(&self, previous_name: &str, new_name: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        let previous_branch_path = helpers::get_branch_path(previous_name);

        if !helpers::check_if_file_exists(&previous_branch_path, path_handler) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "No branch with the specified name was found",
            )));
        }

        let new_branch_path = path_handler.get_relative_path(&helpers::get_branch_path(new_name));

        fs::rename(
            path_handler.get_relative_path(&previous_branch_path),
            new_branch_path,
        )?;

        if Head::get_current_branch_name(path_handler)? == previous_name {
            Head::change_head_branch(new_name, path_handler)?
        }

        Ok(())
    }
}

/// Implementation of the `Command` trait for the `Branch` type.
///
/// This implementation defines the behavior of the `execute` method for the `Branch` type.
///
/// # Arguments
///
/// * `head` - A mutable reference to the `Head` instance representing the Git repository's head.
/// * `args` - An optional slice of string references representing command-line arguments.
///
/// # Command Handling
///
/// The `execute` method interprets the provided command-line arguments and performs the following actions:
///
/// * If no arguments are provided (`args` is `None`), it lists all branches in the repository.
/// * If the "-d" flag is provided in the arguments, it sets the `delete_flag` to `true` and attempts to delete a branch.
/// * If the "-m" flag is provided in the arguments, it sets the `rename_flag` to `true`.
/// * All other arguments are treated as branch names, and the code populates `first_branch_name` and `second_branch_name` options.
///
/// If `first_branch_name` is `None`, it is populated with the first encountered branch name. If `first_branch_name` is already populated, `second_branch_name` is populated with the next encountered branch name. This ensures that `second_branch_name` is only populated after `first_branch_name`.
///
/// Based on the above flags and branch names, the method takes appropriate actions such as printing all branches, deleting a branch, renaming a branch, or creating a new branch. The behavior has been updated to handle different combinations of flags and branch names.
///
/// # Errors
///
/// If any errors occur during branch operations, they are returned as `Result` with an associated error type implementing the `Error` trait.
///
/// # Examples
///
/// rust
/// let mut head = Head::new(); // Initialize a Head instance.
/// let args1 = Some(&["my-branch1"]); // Command-line arguments.
/// let result1 = Branch.execute(args1);
/// assert!(result1.is_ok());
/// let args2 = Some(&["-d", "my-branch1", "-m", "my-branch2"]); // Command-line arguments.
/// let result2 = Branch.execute(args2);
/// assert!(result2.is_ok());
///
impl Command for Branch {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let list_branches_flag = args.is_none();
        let mut delete_flag = false;
        let mut rename_flag = false;
        let mut first_branch_name: Option<String> = None;
        let mut second_branch_name: Option<String> = None;
        let arg_slice = args.unwrap_or_default();

        for arg in arg_slice {
            // Note the & in for &arg
            match arg {
                DELETE_FLAG => delete_flag = true,
                RENAME_FLAG => rename_flag = true,
                _ => {
                    if first_branch_name.is_none() {
                        first_branch_name = Some(arg.to_string());
                    } else if second_branch_name.is_none() {
                        second_branch_name = Some(arg.to_string());
                    }
                }
            }
        }

        /*
            - if there are no args, print list of branches
            - if there is "-d" flag, and a branch name, delete it
            - if there is "-m" flag, and there isn't "-d" flag, and 2 branch names, rename the "first branch name" to the "second branch name"
            - if there is no flags and a branch name, create a branch with that name
        */

        let mut result = String::new();

        match (
            list_branches_flag,
            delete_flag,
            rename_flag,
            first_branch_name,
            second_branch_name,
        ) {
            (true, _, _, _, _) => result = self.list_all_branches(path_handler)?,
            (_, true, _, Some(name), _) => self.delete_branch(&name, path_handler)?,
            (_, false, true, Some(old_name), Some(new_name)) => {
                self.rename_branch(&old_name, &new_name, path_handler)?
            }
            (false, false, false, Some(name), _) => self.create_new_branch(&name, path_handler)?,
            _ => {}
        }
        Ok(result)
    }
}

pub struct Checkout;

impl Default for Checkout {
    fn default() -> Self {
        Self::new()
    }
}

impl Checkout {
    pub fn new() -> Self {
        Checkout {}
    }
}

impl Command for Checkout {
    /// Executes the `git checkout` command, which changes the current branch to the specified one.
    /// It updates the `HEAD` file to point to the new branch if it's different from the current branch.
    /// If successful, it returns an empty string; otherwise, it returns an error message.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        match args {
            Some(args) => {
                let branch_name = args[0];
                if !helpers::check_if_file_exists(&helpers::get_branch_path(branch_name), path_handler) {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Name did not match any known branch",
                    )));
                }
                if Head::get_current_branch_name(path_handler)? == branch_name {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Already on specified branch",
                    )));
                }
                Head::change_head_branch(branch_name, path_handler)?;
                let head_commit = Head::get_head_commit(path_handler)?;
                WorkingDirectory::clean_working_directory(path_handler)?;
                fs::File::create(path_handler.get_relative_path(INDEX_FILE))?;
                if !head_commit.is_empty() {
                    let head_tree = helpers::get_commit_tree(&head_commit, path_handler)?;
                    WorkingDirectory::update_working_directory_to(&head_tree, path_handler)?;
                    let working_tree = helpers::reconstruct_working_tree(head_commit, path_handler)?;
                    StagingArea::new().change_index_file(working_tree, Vec::new(), path_handler)?;
                }
                println!("Switched to branch {}", branch_name);
            }
            None => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "No branch name was provided",
                )))
            }
        }
        
        Ok(String::new())
    }
}

pub struct CatFile;

impl Default for CatFile {
    fn default() -> Self {
        Self::new()
    }
}

impl CatFile {
    pub fn new() -> Self {
        CatFile {}
    }
}

impl Command for CatFile {
    /// Executes the `cat-file` command, which displays information about a Git object's type or size.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        match args {
            Some(args) => {
                let file = fs::File::open(path_handler.get_relative_path(
                    &helpers::get_object_path(args[1]),
                ))?;

                let mut decoder = Decoder::new(file)?;
                let mut header = Vec::new();

                loop {
                    let mut byte = [0; 1];
                    decoder.read_exact(&mut byte)?;

                    // Check if the byte is '\0'
                    if byte[0] == b'\0' {
                        break; // Exit the loop if null byte is encountered
                    }
                    header.push(byte[0]);
                }

                let header_str = String::from_utf8(header)?;
                let parts: Vec<&str> = header_str.trim_end().split(' ').collect();

                match args[0] {
                    TYPE_FLAG => println!("{}", parts[0]),
                    SIZE_FLAG => println!("{}", parts[1]),
                    _ => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Flag error"))),
                }
            }
            None => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "No arguments received",
                )))
            }
        }

        Ok(String::new())
    }
}

pub struct HashObject;

impl Default for HashObject {
    fn default() -> Self {
        Self::new()
    }
}

impl HashObject {
    pub fn new() -> Self {
        HashObject {}
    }
}

impl Command for HashObject {
    /// Executes the `hash-object` command, which calculates the hash of a given file or data.
    /// If the write flag is specified, the object is created as a file in the objects subdirectory.
    /// Default object type is "blob" but can be specified with type flag.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        if args.is_none() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "No path provided",
            )));
        }
        let arg_slice = args.unwrap_or_default();
        let mut path: &str = "";
        let mut obj_type = ObjectType::Blob;
        let mut write = false;
        let mut awaiting_type = false;
        for &item in &arg_slice {
            match item {
                TYPE_FLAG => {
                    awaiting_type = true;
                }
                WRITE_FLAG => write = true,
                _ => {
                    if awaiting_type {
                        if let Some(new_obj_type) = ObjectType::new(item) {
                            obj_type = new_obj_type;
                        } else {
                            eprintln!("Unknown object type for input: {}", item);
                            return Ok(String::new());
                        }
                        awaiting_type = false;
                    } else {
                        path = item
                    }
                }
            }
        }
        if path.is_empty() {
            eprintln!("Please provide a file path or data to hash.");
            return Ok(String::new());
        }
        let content = helpers::read_file_content(path)?;
        let object_hash;
        if write {
            let file_len = helpers::get_file_length(path)?;
            return HashObjectCreator::write_object_file(content, obj_type, file_len, path_handler);
        } else {
            object_hash = helpers::generate_sha1_string(content.as_str());
            println!("{}", object_hash);
        }
        Ok(object_hash)
    }
}

pub struct Commit {
    stg_area: StagingArea,
}

impl Default for Commit {
    fn default() -> Self {
        Self::new()
    }
}

impl Commit {
    pub fn new() -> Self {
        Commit {
            stg_area: StagingArea::new(),
        }
    }

}

impl Command for Commit {
    /// Executes the `commit` command, creating a new commit for the changes in the staging area.
    /// To achieve this, it creates a "tree" which is the index file turned into a tree object.
    /// Then it creates a commit file, which contains the tree object hash, the commit's parent
    /// commits and the given message with the message flag.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        if helpers::get_file_length(&path_handler.get_relative_path(INDEX_FILE))? == 0 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "No changes staged for commit",
            )));
        }

        let mut message: Option<&str> = None;
        let mut message_flag = false;
        let arg_slice = args.unwrap_or_default();

        for arg in arg_slice {
            match arg {
                MESSAGE_FLAG => message_flag = true,
                _ => {
                    if message_flag {
                        message = Some(arg)
                    }
                }
            }
        }
        message = if message_flag { message } else { None };
        let head_commit = Head::get_head_commit(path_handler)?;
        let mut parent = Vec::new();
        if !head_commit.is_empty() {
            parent.push(head_commit)
        }
        let commit_object_hash = HashObjectCreator::create_commit_object(message, parent, path_handler)?;

        let _ = helpers::update_branch_hash(&Head::get_current_branch_name(path_handler)?, &commit_object_hash, path_handler);

        self.stg_area.unstage_index_file(path_handler)?;
        Ok(String::new())
    }
}

pub struct Rm {
    stg_area: StagingArea,
}

impl Default for Rm {
    fn default() -> Self {
        Self::new()
    }
}

impl Rm {
    pub fn new() -> Self {
        Rm {
            stg_area: StagingArea::new(),
        }
    }
}

impl Command for Rm {
    /// Receives a file path and removes it from the staging area.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        match args {
            Some(args) => {
                self.stg_area.remove_file(args[0], path_handler)?;
            }
            None => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Did not receive a file path to remove",
                )))
            }
        }
        Ok(String::new())
    }
}

pub struct Add {
    stg_area: StagingArea,
}

impl Default for Add {
    fn default() -> Self {
        Self::new()
    }
}

impl Add {
    pub fn new() -> Self {
        Add {
            stg_area: StagingArea::new(),
        }
    }
}

impl Command for Add {
    /// Receives a file path and adds it to the staging area.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        match args {
            Some(args) => {
                if (CheckIgnore::new().execute(Some(vec![args[0]]), path_handler)?).is_empty() {
                    self.stg_area.add_file(args[0], path_handler)?;
                } else {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: File is included in '.gitignore'",
                    )));
                }
            }
            None => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Did not receive a file path to add",
                )))
            }
        }
        Ok(String::new())
    }
}

pub struct Status;

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

impl Status {
    pub fn new() -> Self {
        Status {}
    }
}

impl Command for Status {
    /// Execute the "status" command to check the status of the Git repository.
    /// This command checks the status of the current Git repository and prints the
    /// status of files in the working directory, indicating whether they are
    /// modified, staged, or unstaged.
    fn execute(&self, _args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        println!("On branch {}", Head::get_current_branch_name(path_handler)?);
        
        let last_commit_hash: String = Head::get_head_commit(path_handler)?;
        let mut tree_content: Vec<(String, String, String)> = Vec::new();
        if !last_commit_hash.is_empty() {
            let tree_hash = helpers::get_commit_tree(&last_commit_hash, path_handler)?;
            tree_content = helpers::read_tree_content(&tree_hash, path_handler)?;
        }

        let index_file_content =
            helpers::read_file_content(&path_handler.get_relative_path(INDEX_FILE))?;
        let index_objects: Vec<String> =
            index_file_content.lines().map(|s| s.to_string()).collect();
        let mut line_result = String::new();
        let mut line = String::new();
        for pos in 0..(index_objects.len()) {
            let index_file_line: Vec<&str> = index_objects[pos].split(';').collect();
            if pos < tree_content.len() {
                let (_, _, hash_string) = tree_content[pos].clone();
                let current_object_content = helpers::read_file_content(index_file_line[0])?;
                let current_object_hash = HashObjectCreator::generate_object_hash(
                    ObjectType::Blob,
                    get_file_length(index_file_line[0])?,
                    &current_object_content,
                );

                if hash_string != index_file_line[1] && index_file_line[2] == "2" {
                    line = format!("modified: {} (Staged)", index_file_line[0]);
                } else if current_object_hash != hash_string && index_file_line[2] == "0" {
                    line = format!("modified: {} (Unstaged)", index_file_line[0]);
                }
            } else {
                line = format!("new file: {} (Staged)", index_file_line[0]);
            }

            if !line.is_empty() {
                println!("{}", line);
                line_result.push_str(&line);
                line_result.push('\n');
            }
        }
        if line_result.is_empty() {
            line = "nothing to commit, working tree clean".to_string();
            line_result.push_str(&line);
            line_result.push('\n');
            println!("{}", line);
        }
        Ok(line_result)
    }
}

pub struct Remote;

impl Default for Remote {
    fn default() -> Self {
        Self::new()
    }
}

impl Remote {
    pub fn new() -> Self {
        Remote {}
    }

    /// Adds a new remote repository configuration to the Git configuration file.
    fn add_new_remote(&self, remote_name: String, url: String, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        let config_content =
            helpers::read_file_content(&path_handler.get_relative_path(CONFIG_FILE))?;

        let section_header = format!("[remote '{}']", remote_name);
        let new_config_content = format!("{}{}\nurl = {}\n", config_content, section_header, url);

        if config_content.contains(&section_header) {
            //en git permite agregar mas de un remote con mismo nombre si su config o url son distintos, me parece que complejiza mucho y por ahora mejor no poder agregar dos de mismo nombre
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Remote already exists in the configuration.",
            )));
        }

        let mut config_file = fs::File::create(path_handler.get_relative_path(CONFIG_FILE))?;
        config_file.write_all(new_config_content.as_bytes())?;

        let remote_dir_path = format!("{}/{}", R_REMOTES, remote_name);
        fs::create_dir(path_handler.get_relative_path(&remote_dir_path))?;

        println!("Added new remote: {}", remote_name);

        Ok(())
    }

    /// Removes a specified remote repository configuration from the Git configuration file.
    fn remove_remote(&self, remote_name: String, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        let config_content =
            helpers::read_file_content(&path_handler.get_relative_path(CONFIG_FILE))?;

        let remote_header = format!("[remote '{}']", remote_name);
        let mut new_config_content = String::new();
        let mut is_inside_remote_section = false;

        for line in config_content.lines() {
            if line == remote_header {
                is_inside_remote_section = true;
            } else if line.starts_with('[') {
                is_inside_remote_section = false;
            }
            if !is_inside_remote_section {
                new_config_content.push_str(line);
                new_config_content.push('\n');
            }
        }

        let mut config_file = fs::File::create(path_handler.get_relative_path(CONFIG_FILE))?;
        config_file.write_all(new_config_content.as_bytes())?;

        let remote_dir = format!("{}/{}", R_REMOTES, remote_name);
        fs::remove_dir_all(path_handler.get_relative_path(&remote_dir))?;

        Ok(())
    }

    /// Lists and prints the names of remote repositories configured in the Git configuration.
    fn list_remotes(&self) -> Result<(), Box<dyn Error>> {
        let config_content = helpers::read_file_content(CONFIG_FILE)?;

        for line in config_content.lines() {
            if line.starts_with("[remote '") {
                let remote_name = line.trim_start_matches("[remote '").trim_end_matches("']");
                println!("{}", remote_name);
            }
        }

        Ok(())
    }
}

impl Command for Remote {
    /// Executes Command for Remote. When no flags are received, all remotes are listed. If the add flag is received
    /// with a name and a new url, a remote is added to the config file. If a remove flag and a name is received,
    /// the remote with said name will be removed from the config file.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        if args.is_none() {
            self.list_remotes()?;
            return Ok(String::new());
        }
        let mut add_flag = false;
        let mut remove_flag = false;
        let mut name = None;
        let mut url = None;
        let arg_slice = args.unwrap_or_default();

        for arg in arg_slice {
            match arg {
                ADD_FLAG => add_flag = true,
                REMOVE_FLAG => remove_flag = true,
                _ => {
                    if name.is_none() {
                        name = Some(arg.to_string());
                    } else if url.is_none() {
                        url = Some(arg.to_string());
                    }
                }
            }
        }

        match (add_flag, remove_flag, name, url) {
            (true, _, Some(name), Some(url)) => self.add_new_remote(name, url, path_handler)?,
            (_, true, Some(name), _) => self.remove_remote(name, path_handler)?,
            _ => {
                if add_flag {
                    println!("To add a new remote specify remote's name and url")
                } else if remove_flag {
                    println!("To remove a remote specify the remote's name")
                }
            }
        }
        Ok(String::new())
    }
}

pub struct PackObjects;

impl Default for PackObjects {
    fn default() -> Self {
        Self::new()
    }
}

impl PackObjects {
    pub fn new() -> Self {
        PackObjects {}
    }

    fn get_tree_objects(
        object_set: &mut HashSet<String>,
        tree_hash: &str,
        path_handler: &PathHandler
    ) -> Result<(), Box<dyn Error>> {
        object_set.insert(tree_hash.to_string());
        let tree_content = helpers::read_tree_content(tree_hash, path_handler)?;

        for (file_mode, _file_name, object_hash) in tree_content {
            match file_mode.as_str() {
                TREE_FILE_MODE => {
                    object_set.insert(object_hash.clone());
                }
                TREE_SUBTREE_MODE => {
                    // object_set.insert(object_hash.clone());
                    PackObjects::get_tree_objects(object_set, &object_hash, path_handler)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn calculate_object_header(&self, object_type: ObjectType, object_size: usize) -> Vec<u8> {
        let mut header = Vec::new();

        // Encode the object type
        let type_byte: u8 = match object_type {
            ObjectType::Commit => 0b0001,
            ObjectType::Tree => 0b0010,
            ObjectType::Blob => 0b0011,
            ObjectType::Tag => 0b0100,
        };

        // Combine type_byte and the last 4 bits of object_size
        let combined_byte = (type_byte << 4) | ((object_size as u8) & 0x0F);

        // Encode the object size in a variable-length format if it requires more than one byte
        let mut size = object_size >> 4;
        if size > 0 {
            header.push(combined_byte | 0x80); // Set the first bit to 1 in the first byte
        } else {
            header.push(combined_byte);
        }

        while size > 0 {
            let byte = (size as u8) & 0x7F;
            size >>= 7;
            header.push(byte);
        }

        // Print the bit representation of the final header
        // print_bits(&header);

        header
    }
}

impl Command for PackObjects {
    /// Execute the `PackObjects` command.
    /// This command generates a Git pack file that contains compressed Git objects.
    /// The pack file format is used to efficiently store objects and their history.
    /// It also creates an index file that helps locate objects in the pack file.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let commit_list = args.unwrap_or_default(); //aca recibo hashes de commits
        // For each hash it looks for a hash tree
        let mut object_set: HashSet<String> = HashSet::new();
        for commit_hash in commit_list {
            object_set.insert(commit_hash.to_string());
            let tree_hash = helpers::get_commit_tree(commit_hash, path_handler)?;
            PackObjects::get_tree_objects(&mut object_set, &tree_hash, path_handler)?;
        }

        // Create an uncompressed pack file
        let mut pack_file_content = Vec::new();

        // List all objects in the .git/objects directory
        let mut object_count: u32 = 0;
        // Iterate through objects
        for object_hash in object_set {
            // going through hashes in objects_list
            object_count += 1;

            let (object_type, object_content, object_size) =
                helpers::read_object_to_bytes(object_hash.to_string(), path_handler)?;

            // Append object content to the uncompressed pack file
            let object_size_usize: usize = object_size.parse()?;
            let compressed_content: &Vec<u8> = &helpers::compress_bytes(&object_content)?;
            let header = self.calculate_object_header(object_type, object_size_usize);
            
            pack_file_content.extend_from_slice(&header);
            pack_file_content.extend_from_slice(compressed_content);
        }

        let mut pack_file_final = Vec::new();
        // Generate pack header
        let version = [0u8, 0u8, 0u8, 2u8];
        pack_file_final.extend_from_slice(b"PACK");
        pack_file_final.extend_from_slice(&version);
        pack_file_final.extend_from_slice(&object_count.to_be_bytes());

        pack_file_final.extend_from_slice(&pack_file_content);

        let pack_checksum = calculate_sha1_hash(&pack_file_final);
        let checksum_str = helpers::hex_string_to_bytes(&pack_checksum.clone());

        let pack_file_path = format!(".git/pack/pack-{}.pack", checksum_str);
        
        let mut pack_file = fs::File::create(path_handler.get_relative_path(&pack_file_path))?;

        pack_file.write_all(&pack_file_final)?;
        pack_file.write_all(&pack_checksum)?;

        Ok(helpers::hex_string_to_bytes(&pack_checksum).to_string())
    }
}

fn calculate_sha1_hash(data: &[u8]) -> [u8; 20] {
    // Create a Sha1 object
    let mut sha1 = Sha1::new();

    // Update the hash with the data
    sha1.input(data);

    // Obtain the hash result as a Vec<u8>
    let mut hash_result: [u8; 20] = Default::default();
    sha1.result(&mut hash_result);

    hash_result
}

pub struct UnpackObjects;

impl Default for UnpackObjects {
    fn default() -> Self {
        Self::new()
    }
}

impl UnpackObjects {
    pub fn new() -> Self {
        UnpackObjects {}
    }

    fn keep_bits(value: usize, bits: u8) -> usize {
        value & ((1 << bits) - 1)
    }

    /// Reads a fixed number of bytes from a stream.
    /// Rust's "const generics" make this function very useful.
    fn read_bytes<R: Read, const N: usize>(stream: &mut R) -> io::Result<[u8; N]> {
        let mut bytes = [0; N];
        stream.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    fn read_hash<R: Read>(stream: &mut R) -> io::Result<String> {
        let bytes: [u8; 20] = Self::read_bytes(stream)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// Read 7 bits of data and a flag indicating whether there are more
    fn read_varint_byte<R: Read>(stream: &mut R) -> io::Result<(u8, bool)> {
        let [byte] = Self::read_bytes(stream)?;
        let value = byte & !VARINT_CONTINUE_FLAG;
        let more_bytes = byte & VARINT_CONTINUE_FLAG != 0;
        Ok((value, more_bytes))
    }

    fn read_size_encoding<R: Read>(stream: &mut R) -> io::Result<usize> {
        let mut value = 0;
        let mut length = 0; // the number of bits of data read so far
        loop {
            let (byte_value, more_bytes) = Self::read_varint_byte(stream)?;
            // Add in the data bits
            value |= (byte_value as usize) << length;
            // Stop if this is the last byte
            if !more_bytes {
                return Ok(value);
            }

            length += VARINT_ENCODING_BITS;
        }
    }
    /// Object type and uncompressed pack data size
    /// are stored in a "size-encoding" variable-length integer.
    /// Bits 4 through 6 store the type and the remaining bits store the size.
    fn read_type_and_size<R: Read>(stream: &mut R) -> Result<(u8, usize), Box<dyn Error>> {
        let value = Self::read_size_encoding(stream)?;
        let object_type = Self::keep_bits(value >> TYPE_BYTE_SIZE_BITS, TYPE_BITS) as u8;
        let size = Self::keep_bits(value, TYPE_BYTE_SIZE_BITS)
            | (value >> VARINT_ENCODING_BITS << TYPE_BYTE_SIZE_BITS);
        Ok((object_type, size))
    }

    fn read_offset_encoding<R: Read>(stream: &mut R) -> io::Result<u64> {
        let mut value = 0;
        loop {
            let (byte_value, more_bytes) = Self::read_varint_byte(stream)?;
            // Add the new bits at the *least* significant end of the value
            value = (value << VARINT_ENCODING_BITS) | byte_value as u64;
            if !more_bytes {
                return Ok(value);
            }

            // Increase the value if there are more bytes, to avoid redundant encodings
            value += 1;
        }
    }

    // Read an integer of up to `bytes` bytes.
    // `present_bytes` indicates which bytes are provided. The others are 0.
    fn read_partial_int<R: Read>(
        stream: &mut R,
        bytes: u8,
        present_bytes: &mut u8,
    ) -> io::Result<usize> {
        let mut value = 0;
        for byte_index in 0..bytes {
            // Use one bit of `present_bytes` to determine if the byte exists
            if *present_bytes & 1 != 0 {
                let [byte] = Self::read_bytes(stream)?;
                value |= (byte as usize) << (byte_index * 8);
            }
            *present_bytes >>= 1;
        }
        Ok(value)
    }

    // Reads a single delta instruction from a stream
    // and appends the relevant bytes to `result`.
    // Returns whether the delta stream still had instructions.
    fn apply_delta_instruction<R: Read>(
        stream: &mut R,
        base: &[u8],
        result: &mut Vec<u8>,
    ) -> Result<bool, Box<dyn Error>> {
        // Check if the stream has ended, meaning the new object is done
        let instruction = match Self::read_bytes(stream) {
            Ok([instruction]) => instruction,
            Err(err) if err.kind() == ErrorKind::UnexpectedEof => return Ok(false),
            Err(err) => return Err(Box::new(err)),
        };
        if instruction & COPY_INSTRUCTION_FLAG == 0 {
            // Data instruction; the instruction byte specifies the number of data bytes
            if instruction == 0 {
                // Appending 0 bytes doesn't make sense, so git disallows it
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Invalid Data Instructions",
                )));
            }

            // Append the provided bytes
            let mut data = vec![0; instruction as usize];
            stream.read_exact(&mut data)?;
            result.extend_from_slice(&data);
        } else {
            // Copy instruction
            let mut nonzero_bytes = instruction;
            let offset = Self::read_partial_int(stream, COPY_OFFSET_BYTES, &mut nonzero_bytes)?;
            let mut size = Self::read_partial_int(stream, COPY_SIZE_BYTES, &mut nonzero_bytes)?;
            if size == 0 {
                // Copying 0 bytes doesn't make sense, so git assumes a different size
                size = COPY_ZERO_SIZE;
            }
            // Copy bytes from the base object
            let base_data = base.get(offset..(offset + size)).ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                "Invalid copy instructions",
            ))?;
            result.extend_from_slice(base_data);
        }
        Ok(true)
    }

    fn apply_delta(
        pack_file: &mut fs::File,
        base_object_content: &[u8],
        base_type: ObjectType,
    ) -> Result<(ObjectType, Vec<u8>, usize), Box<dyn Error>> {
        // let Object { object_type, contents: ref base } = *base;
        let mut delta = Decoder::new(pack_file)?; //aca esta mal esta descompresion
        let base_size = Self::read_size_encoding(&mut delta)?;
        if base_object_content.len() != base_size {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Incorrect base object length",
            )));
        }

        let result_size = Self::read_size_encoding(&mut delta)?;
        let mut result = Vec::with_capacity(result_size);
        while Self::apply_delta_instruction(&mut delta, base_object_content, &mut result)? {}
        if result.len() != result_size {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Incorrect object length",
            )));
        }

        // The object type is the same as the base object
        Ok((base_type, result, result_size))
    }

    fn seek(file: &mut fs::File, offset: u64) -> io::Result<()> {
        file.seek(SeekFrom::Start(offset))?;
        Ok(())
    }
    fn read_pack_object(
        pack_file: &mut fs::File,
        offset: u64,
        path_handler: &PathHandler
    ) -> Result<(ObjectType, Vec<u8>, usize), Box<dyn Error>> {
        let (object_type, size) = Self::read_type_and_size(pack_file)?;
        let object_type = match object_type {
            1 => PackObjectType::Base(ObjectType::Commit),
            2 => PackObjectType::Base(ObjectType::Tree),
            3 => PackObjectType::Base(ObjectType::Blob),
            4 => PackObjectType::Base(ObjectType::Tag),
            6 => PackObjectType::OffsetDelta,
            7 => PackObjectType::HashDelta,
            _ => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Invalid Object Type",
                )))
            }
        };
        match object_type {
            PackObjectType::Base(object_type) => {
                // The object contents are zlib-compressed
                let mut contents = Vec::with_capacity(size);
                Decoder::new(pack_file)?.read_to_end(&mut contents)?;
                if contents.len() != size {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Incorrect object size",
                    )));
                }

                Ok((object_type, contents, size))
            }
            PackObjectType::OffsetDelta => {
                let delta_offset = Self::read_offset_encoding(pack_file)?;
                let base_offset = offset.checked_sub(delta_offset).ok_or(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Invalid OffsetDelta offset",
                ))?;
                let (base_object_type, base_object_content, _size) =
                    Self::read_pack_object(pack_file, base_offset, path_handler)?;
                Self::seek(pack_file, offset)?;
                Self::apply_delta(pack_file, &base_object_content, base_object_type)
            }
            PackObjectType::HashDelta => {
                let hash = Self::read_hash(pack_file)?; // esto lo tengo que ver como implementar yo. seria la lectura del hash del delta object
                let (object_type, base_object_content, _) = helpers::read_object_to_string(hash, path_handler)?; // aca como hace referencia a un objecto base, ya va a tener que estar descomprimido
                return Self::apply_delta(pack_file, base_object_content.as_bytes(), object_type);
            }
        }
    
    }
}

impl Command for UnpackObjects {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        // Extract arguments or default to an empty vector
        let arg_slice = args.unwrap_or_default();

        // Open the specified pack file
        let mut pack_file = fs::File::open(arg_slice[0])?;

        // Retrieve the size of the pack file
        let _pack_file_size = helpers::get_file_length(arg_slice[0])?;

        // Read the header of the pack file
        let mut header = vec![0; 12]; //Size of header is fixed
        pack_file.read_exact(&mut header)?;

        // Extract the number of objects from the header
        let object_amount = u32::from_be_bytes(header[8..12].try_into()?);

        // Initialize offset after the header
        let mut offset: u64 = 12; //Skipping the header
        let mut objects_unpacked = 1;

        // Iterate over each object in the pack file
        while objects_unpacked <= object_amount {
            // Read the pack object at the specified offset
            let (object_type, content, size) = Self::read_pack_object(&mut pack_file, offset, path_handler)?;

            // Convert content to a string (not used, can be removed if unnecessary)
            let _content_to_string = String::from_utf8_lossy(&content).to_string();

            // Write the object content to a file
            HashObjectCreator::write_object_file_bytes(&content, object_type, size, path_handler)?;

            // Update offset for the next object
            offset += size as u64;
            objects_unpacked += 1;
        }

        // Return an empty string indicating successful execution
        Ok(String::new())
    }
}

pub struct Fetch;

impl Default for Fetch {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetch {
    /// Creates a new `Push` instance.
    pub fn new() -> Self {
        Fetch {}
    }

    pub fn add_remote_ref(
        &self,
        ref_hash: &str,
        ref_name: &str,
        remote_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let split_ref_name: Vec<&str> = ref_name.split('/').collect(); 
        let remote_ref_name = split_ref_name[2];
        let mut ref_path = String::new();
    
        match split_ref_name[1] {
            "heads" => {
                let dir_path = format!("{}/{}", R_REMOTES, remote_name);
                if !helpers::check_if_directory_exists(&dir_path.clone()) {
                    fs::create_dir(dir_path.clone())?;
                }
                ref_path = format!("{}/{}", dir_path, remote_ref_name);
            }

            "tags" => ref_path = format!("{}/{}", R_TAGS, remote_ref_name),
            _ => {}
        }
        
        let mut ref_file = fs::File::create(ref_path)?;
        ref_file.write_all(ref_hash.as_bytes())?;
        Ok(())
    }

}

impl Command for Fetch {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let remote_url;
        let mut remote_name = DEFAULT_REMOTE_REPOSITORY;
        match args {
            Some(args) => match helpers::get_remote_url(args[0]) {
                Ok(url) => {
                    remote_url = url;
                    remote_name = args[0];
                }
                Err(_) => remote_url = args[0].to_string(),
            },
            None => {
                remote_url = helpers::get_remote_url(DEFAULT_REMOTE_REPOSITORY)?;
            }
        }

        let refs = client::client_protocol::ClientProtocol::new()
            .fetch_from_remote_with_our_server(remote_url, path_handler)?;
        UnpackObjects::new().execute(Some(vec![&path_handler.get_relative_path(RECEIVED_PACK_FILE)]), path_handler)?;
        for (ref_hash, ref_name) in refs {
            
            self.add_remote_ref(&ref_hash, &ref_name, remote_name)?;
        }


        Ok(String::new())
    }
}

pub struct Push;

impl Default for Push {
    fn default() -> Self {
        Self::new()
    }
}

impl Push {
    /// Creates a new `Push` instance.
    pub fn new() -> Self {
        Push {}
    }
}

impl Command for Push {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let mut remote_url = String::new();
        if Head::get_head_commit(path_handler)?.is_empty(){
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Can not push without a commit",
            )))
        }
        let mut _remote_name = DEFAULT_REMOTE_REPOSITORY;
        let _branch = Head::get_current_branch_name(path_handler)?;
        if let Some(args) = args {
            match helpers::get_remote_url(args[0]) {
                Ok(url) => {
                    remote_url = url;
                    _remote_name = args[0];
                    
                }
                Err(_) => {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Name is not a remote",
                    )))
                }
            }
        }
        if remote_url.is_empty() {
            remote_url = helpers::get_remote_url(DEFAULT_REMOTE_REPOSITORY)?;
        }
        ClientProtocol::new().receive_pack(remote_url.to_string(), path_handler)?;

        Ok(String::new())
    }
}

pub struct Pull;

impl Default for Pull {
    fn default() -> Self {
        Self::new()
    }
}

impl Pull {
    pub fn new() -> Self {
        Pull {}
    }
}

impl Command for Pull {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let remote_url;
        let mut remote_name = DEFAULT_REMOTE_REPOSITORY;
        match args {
            Some(args) => match helpers::get_remote_url(args[0]) {
                Ok(url) => {
                    remote_url = url;
                    remote_name = args[0];
                }
                Err(_) => remote_url = args[0].to_string(),
            },
            None => {
                remote_url = helpers::get_remote_url(DEFAULT_REMOTE_REPOSITORY)?;
            }
        }
        Fetch::new().execute(Some(vec![&remote_url]), path_handler)?;
        let remote_branch = format!("{}/{}", remote_name, Head::get_current_branch_name(path_handler)?);
        
        if !helpers::check_if_file_exists(&format!("{}/{}", R_REMOTES, remote_branch), path_handler) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: There is no tracking information for the current branch.",
            )));
        }
        Merge::new().execute(Some(vec![&remote_branch]), path_handler)?;
        Ok(String::new())
    }
}

pub struct Clone;

impl Default for Clone {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone {
    /// Creates a new `Clone` instance.
    pub fn new() -> Self {
        Clone {}
    }
}

impl Command for Clone {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        Init::new().execute(None, path_handler)?;

        match args {
            Some(remote_repository) => {
                Remote::new().execute(Some(vec!["add", "origin", remote_repository[0]]), path_handler)?;
                Fetch::new().execute(None, path_handler)?;
                let remote_branches = helpers::get_remote_branches(DEFAULT_REMOTE_REPOSITORY, path_handler)?;
                helpers::update_branches(remote_branches, path_handler)?;
                Pull::new().execute(Some(vec!["origin"]), path_handler)?;
            }
            None => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: No remote repository was received to clone",
                )))
            }
        }

        Ok(String::new())
    }
}

/// This module defines the `Log` struct, which is responsible for implementing the "git log" command.
/// It provides methods to generate log entries and execute the command.

pub struct Log;

impl Default for Log {
    fn default() -> Self {
        Self::new()
    }
}

impl Log {
    /// Creates a new `Log` instance.
    pub fn new() -> Self {
        Log {}
    }

    /// Generates log entries for a given base commit and stores them in the provided `entries` vector.
    /// If the base commit ID is too short, it returns an error.
    ///
    /// # Arguments
    ///
    /// * `entries` - A mutable reference to a vector to store log entries.
    /// * `base_commit` - The base commit ID to start generating logs from./// # Returns
    ///
    /// A `Result` containing the execution result or an error message.
    pub fn generate_log_entries(
        entries: &mut Vec<(String, String)>,
        base_commit: String,
        path_handler: &PathHandler
    ) -> Result<String, Box<dyn Error>> {
        if base_commit.len() < 4 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Invalid Commit ID. It's too short",
            )));
        }

        let current_commit = if base_commit == HEAD {
            Head::get_head_commit(path_handler)?
        } else {
            base_commit
        };
        if entries.iter().any(|(key, _)| key == &current_commit) {
            // don't process it again
            return Ok(String::new());
        }

        let (object_type, commit_file_content, _) =
            helpers::read_object_to_string(current_commit.clone(), path_handler)?;

        if object_type != ObjectType::Commit {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Invalid SHA-1. Is not a commit",
            )));
        }
        
        let commit_lines: Vec<String> = commit_file_content
            .split('\n')
            .map(|s| s.to_string())
            .collect();

        let parent_commit_split_line: Vec<String> = commit_lines[1]
            .split_whitespace()
            .map(String::from)
            .collect();

        let mut message = String::new();
        if parent_commit_split_line[0] != PARENT {
            //root commit
            message.push_str(&commit_lines[1..].join("\n"));
            entries.push((current_commit, message));
            return Ok(String::new());
        }
        
        let parent_commit_trimmed = &parent_commit_split_line[1]; 
        message.push_str(&commit_lines[2..].join("\n"));

        entries.push((current_commit, message));

        Log::generate_log_entries(entries, parent_commit_trimmed.clone(), path_handler)?;
        
        Ok(String::new())
    }
}

impl Command for Log {
    /// Executes the "git log" command.
    ///
    /// # Arguments
    ///
    /// * `args` - An optional slice of arguments passed to the command.
    ///
    /// # Returns
    ///
    /// A `Result` containing the execution result or an error message.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        // Extract the arguments from the provided slice or use an empty slice if none is provided
        let empty_args = args.is_none();
        let arg_slice = args.unwrap_or_default();

        // Initialize vectors to store log entries (included and excluded)
        let mut log_entries = Vec::new();
        let mut log_entries_excluded = Vec::new();
        
        // Iterate through the provided arguments
        for arg in arg_slice {
            // Note the & in for &arg
            // Check the first character of each argument
            if let Some(first_char) = arg.chars().next() {
                match first_char {
                    EXCLUDE_LOG_ENTRY => {
                        // Generate log entries for exclusion and store them in the excluded entries vector
                        Log::generate_log_entries(&mut log_entries_excluded, arg[1..].to_string(), path_handler)?;
                        
                    }
                    _ => {
                        // Generate log entries for inclusion and store them in the included entries vector
                        Log::generate_log_entries(&mut log_entries, arg.to_string(), path_handler)?;
                        
                    }
                }
            }
        }

        if empty_args {
            Log::generate_log_entries(&mut log_entries, HEAD.to_string(), path_handler)?;
        }

        // Filter out log entries that are in the excluded entries vector
        log_entries = log_entries
            .iter()
            .filter(|(commit, _)| !log_entries_excluded.iter().any(|(key, _)| key == commit))
            .cloned()
            .collect::<Vec<(String, String)>>();

        // Display the resulting log entries
        for (commit, message) in &log_entries {
            println!(
                "{}commit {}{}\n{}",
                COLOR_YELLOW_CODE, commit, COLOR_RESET_CODE, message
            );
        }
        let result: String =
            log_entries
                .into_iter()
                .fold(String::new(), |mut acc, (key, value)| {
                    writeln!(acc, "{} {}\n", key, value).expect("Error writing to String");
                    acc
                });
        Ok(result)
    }
}
pub struct LsTree;

impl Default for LsTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LsTree {
    
    pub fn new() -> Self {
        LsTree {}
    }

    pub fn generate_tree_entries(
        entries: &mut Vec<String>,
        tree_hash: String,
        direct_flag: bool,
        recurse_flag: bool,
        long_flag: bool,
        path_handler: &PathHandler
    ) -> Result<(), Box<dyn Error>> {
        let current_hash = if tree_hash == HEAD {
            helpers::get_commit_tree(Head::get_head_commit(path_handler)?.as_str(), path_handler)?
        } else {
            tree_hash
        };
        
        let mut tree_content = helpers::read_tree_content(&current_hash, path_handler)?;
        
        if tree_content.is_empty() {
            return Ok(());
        }


        for (file_mode, file_name, object_hash) in &mut tree_content {

            let mut line: String = format!("{} {} {}", file_mode, object_hash, file_name);

            if file_mode == TREE_FILE_MODE && direct_flag {
                // don't add files to the entries if direct flag is on
                continue;
            }

            if long_flag {
                // add size to the line
                let (_, _object_content, object_size) =
                    helpers::read_object_to_string(object_hash.clone(), path_handler)?;
                line.push(' ');
                line.push_str(object_size.as_str());
            }

            if file_mode == TREE_FILE_MODE {
                // add
                entries.push(line.clone());
                continue;
            }

            if file_mode == TREE_SUBTREE_MODE {
                // if direct or not recursive add
                if direct_flag || !recurse_flag {
                    entries.push(line.clone());
                }
                // if recursive loop
                if recurse_flag {
                    return LsTree::generate_tree_entries(
                        entries,
                        object_hash.clone(),
                        direct_flag,
                        recurse_flag,
                        long_flag,
                        path_handler
                    );
                }
            }
        }
        Ok(())
    }
}

impl Command for LsTree {
    /// Executes the "git ls-tree" command.
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let mut direct_flag = false;
        let mut recurse_flag = false;
        let mut long_flag = false;
        let mut tree_entries: Vec<String> = Vec::new();
        let mut tree_hash: Option<String> = None;
        let arg_slice = args.unwrap_or_default();

        for arg in arg_slice {
            // Note the & in for &arg
            match arg {
                DIRECT_FLAG => direct_flag = true,
                RECURSE_FLAG => recurse_flag = true,
                LONG_FLAG => long_flag = true,
                _ => tree_hash = Some(arg.to_string()),
            }
        }

        if !direct_flag && !recurse_flag && !long_flag {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: ls-tree wrong arguments received. supported flags are -d -r -l followed by a tree hash",
            )));
        }

        if let Some(tree) = tree_hash {
            LsTree::generate_tree_entries(
                &mut tree_entries,
                tree,
                direct_flag,
                recurse_flag,
                long_flag,
                path_handler
            )?;
        } else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: ls-tree wrong arguments received. supported flags are -d -r -l followed by a tree hash",
            )));
        }

        for entry in tree_entries {
            println!("{}", entry);
        }

        // Return a successful result (an empty string in this case)
        Ok(String::new())
    }
}

pub struct LsFiles {
    stg_area: StagingArea,
}

impl Default for LsFiles {
    fn default() -> Self {
        Self::new()
    }
}

impl LsFiles {
    /// Creates a new `LsFiles` instance.
    pub fn new() -> Self {
        LsFiles {
            stg_area: StagingArea::new(),
        }
    }
}

impl Command for LsFiles {
    /// Execute the LSFILES command
    ///
    /// This command retrieves and prints file entries based on the specified flags and options.
    /// It supports flags such as DELETE_FLAG, CACHED_FLAG, STAGE_FLAG, MODIFIED_FLAG, and IGNORE_FLAG.
    ///
    /// # Arguments
    ///
    /// * `_head`: A mutable reference to the Git repository's `Head`.
    /// * `args`: An optional vector of string slices representing command-line arguments and flags.
    ///            Supported flags: DELETE_FLAG, CACHED_FLAG, STAGE_FLAG, MODIFIED_FLAG, IGNORE_FLAG.
    ///
    /// # Returns
    ///
    /// A `Result` containing a string or an error. In case of success, the string is empty.
    /// In case of an error, a `Box<dyn Error>` is returned with details about the error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Create an instance of the LSFilesCommand
    /// let lsfiles_command = LsFiles::new();
    ///
    /// // Execute the LSFILES command with specific flags
    /// let result = lsfiles_command.execute(&mut head, Some(vec!["-c"]));
    /// assert!(result.is_ok());
    /// ```
    fn execute(&self, args: Option<Vec<&str>>, _path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let mut file_entries: HashSet<String> = HashSet::new();
        let whole_index_flag = args.is_none();
        let arg_slice = args.unwrap_or_default();

        for arg in arg_slice {
            match arg {
                DELETE_FLAG | CACHED_FLAG | STAGE_FLAG | MODIFIED_FLAG => {
                    let state = match arg {
                        DELETE_FLAG => IndexFileEntryState::Deleted,
                        CACHED_FLAG => IndexFileEntryState::Cached,
                        STAGE_FLAG => IndexFileEntryState::Staged,
                        MODIFIED_FLAG => IndexFileEntryState::Modified,
                        _ => unreachable!(), // This should not happen
                    };

                    let entries = self.stg_area.get_entries_index_file(state)?;
                    for entry in entries {
                        file_entries.insert(entry);
                    }
                }
                IGNORE_FLAG => {
                    let file = fs::File::open(".gitignore.txt")?;
                    let reader = io::BufReader::new(file);
                    for line in reader.lines() {
                        let line = line?;
                        file_entries.insert(line);
                    }
                }
                _ => { /* ignore invalid flags */ }
            }
        }

        if whole_index_flag {
            let entries = self
                .stg_area
                .get_entries_index_file(IndexFileEntryState::Cached)?;
            for entry in entries {
                file_entries.insert(entry);
            }
        }

        for entry in file_entries {
            println!("{:?}", entry);
        }
        // Return a successful result (an empty string in this case)
        Ok(String::new())
    }
}

pub struct CheckIgnore;

impl Default for CheckIgnore {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckIgnore {
    pub fn new() -> Self {
        CheckIgnore {}
    }
}

impl Command for CheckIgnore {
    /// Execute the command, checking for the existence of a file path in the .gitignore file.
    /// Returns a `Result` containing a string. If the file path is found in the .gitignore file,
    /// the path is returned; otherwise, an empty string is returned. Errors are wrapped
    /// in the `Result` type.
    fn execute(&self, args: Option<Vec<&str>>, _path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        //Checking if a .gitignore file exists
        if fs::metadata(".gitignore.txt").is_err() {
            return Ok(String::new());
        }

        // Extract the arguments from the provided slice or use an empty slice if none is provided
        let arg_slice = args.unwrap_or_default();
        let file_path = arg_slice[0];

        let file = fs::File::open(".gitignore.txt")?;
        let reader = io::BufReader::new(file);

        let line_exists = reader
            .lines()
            .any(|line| line.map_or(false, |l| file_path.starts_with(&l)));
        if line_exists {
            println!("{}", file_path);
            return Ok(file_path.to_string());
        }

        Ok(String::new())
    }
}

pub struct Tag;

impl Default for Tag {
    fn default() -> Self {
        Self::new()
    }
}

impl Tag {
    pub fn new() -> Self {
        Tag {}
    }

    fn list_all_tags(&self) -> Result<(), Box<dyn Error>> {
        // Read the contents of the directory
        let entries = fs::read_dir(R_TAGS)?;

        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy(); // Convert to a String

            println!("{}", file_name_str);
        }

        Ok(())
    }

    fn add_new_lightweight_tag(&self, name: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        let last_commit = Head::get_head_commit(path_handler)?;

        let tag_path = format!("{}/{}", R_TAGS, name);
        let mut tag_file = fs::File::create(path_handler.get_relative_path(&tag_path))?;

        tag_file.write_all(last_commit.as_bytes())?;

        Ok(())
    }


    fn delete_tag(&self, name: &str, path_handler: &PathHandler) -> Result<(), Box<dyn Error>> {
        let tag_path = format!("{}/{}", R_TAGS, name);
        fs::remove_file(path_handler.get_relative_path(&tag_path))?;
        Ok(())
    }
}

impl Command for Tag {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        if args.is_none() {
            self.list_all_tags()?;
            return Ok(String::new());
        }

        let mut verify_flag = false;
        let mut delete_flag = false;
        let mut list_flag = false;
        let mut name = None;
        let arg_slice = args.unwrap_or_default();

        for arg in arg_slice {
            match arg {
                LIST_FLAG => list_flag = true,
                VERIFY_FLAG => verify_flag = true,
                DELETE_FLAG => delete_flag = true,
                _ => name = Some(arg),
            }
        }

        match (verify_flag, delete_flag, list_flag, name) {
            (false, false, false, Some(name)) => self.add_new_lightweight_tag(name, path_handler)?,
            (_, true, _, Some(name)) => self.delete_tag(name, path_handler)?,
            (_, _, true, _) => self.list_all_tags()?,
            _ => {}
        }

        Ok(String::new())
    }
}

pub struct ShowRef;

impl Default for ShowRef {
    fn default() -> Self {
        Self::new()
    }
}

impl ShowRef {
    pub fn new() -> Self {
        ShowRef {}
    }

    fn show_refs_in_directory(
        &self,
        directory_entries: ReadDir,
        partial_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        for entry in directory_entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy(); // Convert to a String

            let content = std::fs::read_to_string(entry.path())?;

            println!("{} {}{}", content, partial_path, file_name_str);
        }
        Ok(())
    }
}

impl Command for ShowRef {
    fn execute(&self, _args: Option<Vec<&str>>, _path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        // Read the contents of the directory
        let branch_entries = fs::read_dir(R_HEADS)?;
        self.show_refs_in_directory(branch_entries, "refs/heads/")?;

        let tags_entries = fs::read_dir(R_TAGS)?;
        self.show_refs_in_directory(tags_entries, "refs/tags/")?;

        Ok(String::new())
    }
}

pub struct Merge;

impl Default for Merge {
    fn default() -> Self {
        Self::new()
    }
}

impl Merge {
    pub fn new() -> Self {
        Merge {}
    }
}

impl Command for Merge {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let arg_slice = args.unwrap_or_default();

        if let Some(arg) = arg_slice.first() {
            if *arg == CONTINUE_FLAG {
                if helpers::check_if_conflict_has_been_solved(path_handler).is_err() {
                    println!("Automatic merge failed; fix conflicts and then commit the result");
                    return Ok(String::new())
                }
                if !helpers::check_if_file_exists(&path_handler.get_relative_path(MERGE_HEAD), path_handler) {
                    println!("There is no unresolved merge to continue.");
                    return Ok(String::new())
                }

                let merging_hash = helpers::read_file_content(&path_handler.get_relative_path(MERGE_HEAD))?;
                println!("merging hash: {}", merging_hash);
                fs::remove_file(path_handler.get_relative_path(MERGE_HEAD))?;
                
                let new_commit_hash = helpers::create_merged_working_tree(Head::get_head_commit(path_handler)?, merging_hash, path_handler)?;
                return Ok(new_commit_hash)
            }
        }

        let mut head_commit = Head::get_head_commit(path_handler)?;
        let branch_to_merge = arg_slice[0];
        if arg_slice.len() == 2 {
            let current_branch = arg_slice[1];
            if branch_to_merge == current_branch {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Cannot merge same branch.",
                )))
            }
            helpers::check_if_branch_exists(current_branch, path_handler)?;
            head_commit = helpers::get_branch_last_commit(&helpers::get_branch_path(current_branch), path_handler)?;
        }

        if branch_to_merge == Head::get_current_branch_name(path_handler)? {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Cannot merge same branch.",
            )))
        }

        let mut branch_to_merge_path = helpers::get_branch_path(branch_to_merge);
        if !helpers::check_if_file_exists(&branch_to_merge_path, path_handler) {
            // This means the branch is a remote branch
            branch_to_merge_path = format!("{}/{}", R_REMOTES, branch_to_merge);
        }
        let merging_commit_hash = helpers::get_branch_last_commit(&branch_to_merge_path, path_handler)?;

        if helpers::determine_new_working_tree(head_commit.clone(), merging_commit_hash.clone(), path_handler).is_err() {
            let mut merge_head = fs::File::create(path_handler.get_relative_path(MERGE_HEAD))?;
            merge_head.write_all(merging_commit_hash.as_bytes())?;
            return Ok(String::new())
        }

        let new_commit_hash = helpers::create_merged_working_tree(head_commit, merging_commit_hash, path_handler)?;

        Ok(new_commit_hash)
    }
}



pub struct Rebase;

impl Default for Rebase {
    fn default() -> Self {
        Self::new()
    }
}

impl Rebase {
    pub fn new() -> Self {
        Rebase {}
    }
}

impl Command for Rebase {
    fn execute(&self, args: Option<Vec<&str>>, path_handler: &PathHandler) -> Result<String, Box<dyn Error>> {
        let mut rebasing_branch_name = String::new();
        let rebasing_commit ;
        
        match args {
            Some(arg) => {
                if arg[0] == CONTINUE_FLAG {
                    if !helpers::check_if_file_exists(REBASE_HEAD, path_handler) {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::Other,
                            "Error: No rebase to continue.",
                        )))
                    }
                    rebasing_commit = helpers::read_file_content(REBASE_HEAD)?;

                } else {
                    rebasing_branch_name = arg[0].to_string();
                    helpers::check_if_branch_exists(&rebasing_branch_name, path_handler)?;
                    rebasing_commit = helpers::get_branch_last_commit(&helpers::get_branch_path(&rebasing_branch_name), path_handler)?;
                }
            }
            None => return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: No argument was provided for rebase.",
            ))),
        }

        let mut rebasing_commit_log: Vec<(String, String)> = Vec::new();
        Log::generate_log_entries(&mut rebasing_commit_log, rebasing_commit, path_handler)?;
        let mut previous_commit = Head::get_head_commit(path_handler)?;
        for (commit, _message) in rebasing_commit_log.iter().rev() {
            println!("{}Applying:{} {}", COLOR_RED_CODE, COLOR_RESET_CODE, commit);
            let branch_name = format!("rebase_{}", rebasing_branch_name);
            let _ = Branch::new().create_new_branch(&branch_name, path_handler);
            helpers::update_branch_hash(&branch_name, &commit.clone(), path_handler)?;

            match helpers::determine_new_working_tree(Head::get_head_commit(path_handler)?, commit.clone(), path_handler) {
                Ok(_) => {
                    let _ = Branch::new().execute(Some(vec![DELETE_FLAG, &branch_name]), path_handler)?;
                    let new_commit = HashObjectCreator::create_commit_object(None, vec![previous_commit], path_handler)?;
                    let new_tree = helpers::get_commit_tree(&new_commit, path_handler)?;
                    helpers::update_branch_hash(&Head::get_current_branch_name(path_handler)?, &new_commit, path_handler)?;
                    WorkingDirectory::update_working_directory_to(&new_tree, path_handler)?;
                    previous_commit = new_commit;
                }
                Err(_) => {
                    let mut rebase_head = fs::File::create(path_handler.get_relative_path(REBASE_HEAD))?;
                    rebase_head.write_all(commit.as_bytes())?;
                }
            }
        }
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    use tempfile::tempdir;

    fn common_setup() -> (tempfile::TempDir, String) {
        
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        // Set the environment variable for the relative path

        // Create and execute the Init command
        let init_command = Init::new();
        

        // Check if the Init command was successful

        (temp_dir, temp_path)
    }

    #[test]
    fn test_init_command() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Set the environment variable for relative path
        //env::set_var(RELATIVE_PATH, temp_path);
        let mut path_handler = PathHandler::new(&&temp_path);

        // Create and execute the Init command
        let init_command = Init::new();
        let result = init_command.execute(None, &mut path_handler);

        // Assert that the command executed successfully
        assert!(result.is_ok());

        // Verify the directory structure and necessary files
        assert!(temp_path
            .join(path_handler.get_relative_path(R_HEADS))
            .exists());
        assert!(temp_path
            .join(path_handler.get_relative_path(R_TAGS))
            .exists());
        assert!(temp_path
            .join(path_handler.get_relative_path(OBJECT))
            .exists());
        assert!(temp_path
            .join(path_handler.get_relative_path(PACK))
            .exists());
        assert!(temp_path
            .join(path_handler.get_relative_path(R_REMOTES))
            .exists());
        assert!(temp_path
            .join(path_handler.get_relative_path(CONFIG_FILE))
            .exists());
        // Add more assertions for other files and folders as needed
    }

    #[test]
    fn test_branch_command() {
        let (_temp_dir, _temp_pathh) = common_setup();
        let path_handler = PathHandler::new(_temp_pathh.to_string());
        // Execute the Branch command with various scenarios
        // Example 1: List branches
        let args1 = None;
        let result1 = Branch.execute(args1, &path_handler);
        assert!(result1.is_ok());

        // Example 2: Delete a branch
        let branch_to_delete = Some(vec!["branch_to_delete"]);
        let _ = Branch.execute(branch_to_delete, &path_handler);
        let args2 = Some(vec!["-d", "branch_to_delete"]);
        let result2 = Branch.execute(args2, &path_handler);
        assert!(result2.is_ok());

        // Example 3: Rename a branch
        let new_branch = Some(vec!["old_branch"]);
        let _ = Branch.execute(new_branch, &path_handler);
        let args3 = Some(vec!["-m", "old_branch", "new_branch"]);
        let result3 = Branch.execute(args3, &path_handler);
        assert!(result3.is_ok());

        // Example 4: Create a new branch
        let args4 = Some(vec!["new_branch2"]);
        let result4 = Branch.execute(args4, &path_handler);
        assert!(result4.is_ok());
    }

    #[test]
    fn test_checkout_command() {
        let (_temp_dir, _temp_pathh) = common_setup();
        let path_handler = PathHandler::new(_temp_pathh.to_string());

        // Execute the Checkout command with various scenarios
        // Example 1: Successful checkout
        let args1 = Some(vec!["branch_to_checkout"]);
        let _result4 = Branch.execute(args1.clone(), &path_handler);
        let result1 = Checkout.execute(args1, &path_handler);
        assert!(result1.is_ok());

        // Example 2: Attempt to checkout the same branch (should result in an error)
        let args2 = Some(vec!["branch_to_checkout"]);
        let result2 = Checkout.execute(args2, &path_handler);
        assert!(result2.is_err());

        // Example 3: Attempt to checkout a non-existing branch (should result in an error)
        let args3 = Some(vec!["non_existing_branch"]);
        let result3 = Checkout.execute(args3, panic!());
        assert!(result3.is_err());

        // Example 4: No branch name provided (should result in an error)
    }

    #[test]
    fn test_hashobject_command() {
        let (_temp_dir, _temp_pathh) = common_setup();
        let path_handler = PathHandler::new(_temp_pathh.to_string());

        // Execute the HashObject command with various scenarios
        // Example 1: Calculate hash and print (no write flag)

        let _file = fs::File::create("file.txt");
        let args1 = Some(vec!["file.txt"]);

        let result1 = HashObject.execute(args1, &path_handler);
        assert!(result1.is_ok());

        // Example 2: Calculate hash and write to object file (with write flag)
        let args2 = Some(vec![WRITE_FLAG, "file.txt"]);
        let result2 = HashObject.execute(args2, &path_handler);
        assert!(result2.is_ok());

        // Example 3: Specify object type (blob)
        let args3 = Some(vec![TYPE_FLAG, "blob", "file.txt"]);
        let result3 = HashObject.execute(args3, &path_handler);
        assert!(result3.is_ok());

        // Example 4: No path provided (should result in an error)
        let args5 = None;
        let result5 = HashObject.execute(args5, &path_handler);
        assert!(result5.is_err());
    }

    #[test]
    fn test_catfile_command() {
        let (_temp_dir, _temp_pathh) = common_setup();
        let path_handler = PathHandler::new(_temp_pathh.to_string());

        let _file = fs::File::create("file.txt");
        let args1 = Some(vec![WRITE_FLAG, "file.txt"]);

        let hash_object = HashObject.execute(args1, &path_handler).unwrap();
        // Execute the CatFile command with various scenarios
        // Example 1: Display object type
        let args1 = Some(vec![TYPE_FLAG, &hash_object]);
        let result1 = CatFile.execute(args1, &path_handler);
        assert!(result1.is_ok());

        // Example 2: Display object size
        let args2 = Some(vec![SIZE_FLAG, &hash_object]);
        let result2 = CatFile.execute(args2, &path_handler);
        assert!(result2.is_ok());

        // Example 3: Invalid flag (should result in an error)
        let args3 = Some(vec!["invalid_flag", &hash_object]);
        let result3 = CatFile.execute(args3, &path_handler);
        assert!(result3.is_err());

        // Example 4: No arguments provided (should result in an error)
        let args4 = None;
        let result4 = CatFile.execute(args4, &path_handler);
        assert!(result4.is_err());
    }

    #[test]
    fn test_add_command() {
        // Common setup
        let (_temp_dir, temp_path) = common_setup();
        let path_handler = PathHandler::new(temp_path.to_string());

        // Create a sample file to be added
        let file_path = temp_path.clone() + "/sample.txt";
        fs::write(&file_path, "Sample file content").expect("Failed to create a sample file");

        let add_command = Add::new();

        // Convert &str to String before creating the args vector
        let args: Option<Vec<&str>> = Some(vec![&file_path]);

        let result = add_command.execute(args, &path_handler);

        // Assert that the command executed successfully
        assert!(result.is_ok(), "Add command failed: {:?}", result);

        // Cleanup: The temporary directory will be automatically deleted when temp_dir goes out of scope
    }

    #[test]
    fn test_commit_command() {
        // Common setup
        let (_temp_dir, temp_path) = common_setup();
        let path_handler = PathHandler::new(temp_path.to_string());

        // Create a sample file to be added
        let file_path = temp_path.clone() + "/sample.txt";
        fs::write(&file_path, "Sample file content").expect("Failed to create a sample file");

        // Execute the Add command
        let add_command = Add::new();
        let args_add: Option<Vec<&str>> = Some(vec![&file_path]);
        let _result_add = add_command.execute(args_add, &path_handler);

        // Execute the Commit command
        let commit_command = Commit::new();
        let args_commit: Option<Vec<&str>> = Some(vec!["-m", "Initial commit"]);
        let result_commit = commit_command.execute(args_commit, &path_handler);

        // Assert that the command executed successfully
        assert!(
            result_commit.is_ok(),
            "Commit command failed: {:?}",
            result_commit
        );

        // Cleanup: The temporary directory will be automatically deleted when temp_dir goes out of scope
    }

    #[test]
    fn test_remove_file_from_staging_area() {
        // Common setup
        let (_temp_dir, temp_path) = common_setup();
        let path_handler = PathHandler::new(temp_path.to_string());

        // Create a sample file to be added
        let file_path = temp_path.clone() + "/sample.txt";
        fs::write(&file_path, "Sample file content").expect("Failed to create a sample file");

        // Execute the Add command
        let add_command = Add::new();
        let args_add: Option<Vec<&str>> = Some(vec![&file_path]);
        add_command.execute(args_add, &path_handler).expect("Add command failed");

        // Execute the Rm command
        let rm_command = Rm::new();
        let args_rm: Option<Vec<&str>> = Some(vec![&file_path]);
        let result = rm_command.execute(args_rm, &path_handler);

        // Assert that the command executed successfully
        assert!(result.is_ok(), "Rm command failed: {:?}", result);

        // TODO: Add assertions for the expected state after removal
    }

    #[test]
    fn test_status_command() {
        // Common setup
        let (_temp_dir, temp_path) = common_setup();
        let path_handler = PathHandler::new(temp_path.to_string());

        // Create a sample file in the working directory
        let working_dir_file_path = temp_path.clone() + "sample.txt";
        fs::write(&working_dir_file_path, "Working directory file content")
            .expect("Failed to create a working directory file");

        // Execute the Status command
        let status_command = Status::new();

        let args = None; // You might adjust this based on how your Status command is designed
        let result = status_command.execute(args, &path_handler);

        // Assert that the command executed successfully
        assert!(result.is_ok(), "Status command failed: {:?}", result);

        // Execute the Add command to stage changes
        let add_command = Add::new();
        let args: Option<Vec<&str>> = Some(vec![&working_dir_file_path]);
        let _ = add_command.execute(args, &path_handler);

        // Execute the Commit command to make a commit
        let commit_command = Commit::new();
        let args = Some(vec!["-m", "Test commit message"]);
        let _ = commit_command.execute(args, &path_handler);

        // Execute the Status command
        let args = None; // You might adjust this based on how your Status command is designed
        let result = status_command.execute(args, &path_handler);

        // Assert that the command executed successfully
        assert!(
            result.is_ok(),
            "Status command (With previous commit) failed: {:?}",
            result
        );
        // Clean up: The temporary directory will be automatically deleted when temp_dir goes out of scope
    }

    const REMOTE_NAME: &str = "origin";
    const REMOTE_URL: &str = "127.0.0.1:9418";

    #[test]
    fn test_add_remote() {
        // Common setup
        let _temp_dir = common_setup();
        let path_handler = PathHandler::new(_temp_dir.to_string());

        // Create a new Remote instance
        let remote = Remote::new();

        // Execute the add remote command
        let result = remote.add_new_remote(REMOTE_NAME.to_string(), REMOTE_URL.to_string(), &path_handler);

        // Assert that the command executed successfully
        assert!(result.is_ok(), "Add remote command failed: {:?}", result);

        // Clean up: The temporary directory will be automatically deleted when temp_dir goes out of scope
    }

    #[test]
    fn test_remove_remote() {
        // Common setup
        let _temp_dir = common_setup();
        let path_handler = PathHandler::new(_temp_dir.to_string());

        // Create a new Remote instance
        let remote = Remote::new();

        // Add a remote for testing
        remote
            .add_new_remote(REMOTE_NAME.to_string(), REMOTE_URL.to_string(), &path_handler)
            .unwrap();

        // Execute the remove remote command
        let result = remote.remove_remote(REMOTE_NAME.to_string(), &path_handler);

        // Assert that the command executed successfully
        assert!(result.is_ok(), "Remove remote command failed: {:?}", result);

        // Clean up: The temporary directory will be automatically deleted when temp_dir goes out of scope
    }

    #[test]
    fn test_add_new_lightweight_tag() {
        // Create a temporary directory
        let (_temp_dir, temp_path) = common_setup();
        let path_handler = PathHandler::new(temp_path.to_string());

        // Create a sample file to be added
        let file_path = temp_path.clone() + "/sample.txt";
        fs::write(&file_path, "Sample file content").expect("Failed to create a sample file");

        // Execute the Add command
        let add_command = Add::new();
        let args_add: Option<Vec<&str>> = Some(vec![&file_path]);
        let _result_add = add_command.execute(args_add, &path_handler);

        // Execute the Commit command
        let commit_command = Commit::new();
        let args_commit: Option<Vec<&str>> = Some(vec!["-m", "Initial commit"]);
        let _result_commit = commit_command.execute(args_commit, &path_handler);

        let last_commit = helpers::read_file_content(&(temp_path.clone() + ".git/refs/heads/main"));
        // Create a Tag instance
        let tag = Tag::new();

        // Execute add_new_lightweight_tag
        let _result = tag
            .add_new_lightweight_tag("new_tag", &path_handler)
            .expect("Failed to add new tag");

        // Read the content of the created tag file
        let tag_content = fs::read_to_string(temp_path + ".git/refs/tags/new_tag")
            .expect("Failed to read tag file");

        // Assertions based on tag content
        assert_eq!(tag_content, last_commit.unwrap());
    }

    #[test]
    fn test_delete_tag() {
        // Create a temporary directory
        let (_temp_dir, temp_path) = common_setup();
        let path_handler = PathHandler::new(temp_path.to_string());

        // Create a sample file to be added
        let file_path = temp_path.clone() + "/sample.txt";
        fs::write(&file_path, "Sample file content").expect("Failed to create a sample file");

        // Execute the Add command
        let add_command = Add::new();
        let args_add: Option<Vec<&str>> = Some(vec![&file_path]);
        let _result_add = add_command.execute(args_add, &path_handler);

        // Execute the Commit command
        let commit_command = Commit::new();
        let args_commit: Option<Vec<&str>> = Some(vec!["-m", "Initial commit"]);
        let _result_commit = commit_command.execute(args_commit, &path_handler);

        let _last_commit =
            helpers::read_file_content(&(temp_path.clone() + ".git/refs/heads/main"));
        // Create a Tag instance
        let tag = Tag::new();

        // Execute add_new_lightweight_tag
        let _result = tag
            .add_new_lightweight_tag("new_tag", &path_handler)
            .expect("Failed to add new tag");

        // Execute delete_tag
        tag.delete_tag("new_tag", &path_handler).expect("Failed to delete tag");

        // Check that the tag file is deleted
        assert!(!(Path::new(&(temp_path.clone() + "tags/new_tag"))).exists());
    }

    #[test]
    fn test_check_ignore_file_exists() {
        // Create a temporary directory
        let (_temp_dir, _temp_path) = common_setup();
        let path_handler = PathHandler::new(_temp_path.to_string());

        // Create a .gitignore.txt file in the temporary directory
        let gitignore_path = ".gitignore.txt";
        fs::write(&gitignore_path, "ignored_file.txt")
            .expect("Failed to create .gitignore.txt file");

        // Create a CheckIgnore instance
        let check_ignore = CheckIgnore::new();

        // Execute the check_ignore command
        let result = check_ignore.execute(Some(vec!["ignored_file.txt"]), &path_handler);

        // Assert that the result is the provided file path
        assert_eq!(result.unwrap(), "ignored_file.txt");
        let _ = fs::remove_file("ignored_file.txt");
    }

    #[test]
    fn test_check_ignore_file_not_exists() {
        // Create a temporary directory
        let (_temp_dir,_temp_pathh) = common_setup();
        let path_handler = PathHandler::new(_temp_pathh.to_string());

        // Create a CheckIgnore instance
        let check_ignore = CheckIgnore::new();

        // Execute the check_ignore command with a file that does not exist in .gitignore.txt
        let result = check_ignore.execute(Some(vec!["non_existent_file.txt"]), &path_handler);

        // Assert that the result is an empty string
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_check_ignore_no_gitignore_file() {
        // Create a temporary directory
        let (_temp_dir,_temp_pathh) = common_setup();
        let path_handler = PathHandler::new(_temp_pathh.to_string());

        // Create a CheckIgnore instance
        let check_ignore = CheckIgnore::new();

        // Execute the check_ignore command without a .gitignore.txt file
        let result = check_ignore.execute(Some(vec!["some_file.txt"]), &path_handler);

        // Assert that the result is an empty string
        assert_eq!(result.unwrap(), "");
    }
}
