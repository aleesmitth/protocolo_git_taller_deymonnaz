use std::{fs, error::Error, io, io::Write, io::Read, str};

extern crate libflate;
use libflate::zlib::Decoder;

const OBJECT: &str = ".git/objects";
const PACK: &str = ".git/pack";

const DELETE_FLAG: &str = "-d";
const RENAME_FLAG: &str = "-m";
const TYPE_FLAG: &str = "-t";
const WRITE_FLAG: &str = "-w";
const SIZE_FLAG: &str = "-s";
const MESSAGE_FLAG: &str = "-m";
const EXCLUDE_LOG_ENTRY: char = '^';
const HEAD: &str = "HEAD";
const ADD_FLAG: &str = "add";
const REMOVE_FLAG: &str = "rm";
const R_HEADS: &str = ".git/refs/heads";
const HEAD_FILE: &str = ".git/HEAD";
const R_TAGS: &str = ".git/refs/tags";
const DEFAULT_BRANCH_NAME: &str = "main";
const INDEX_FILE: &str = ".git/index";
const CONFIG_FILE: &str = ".git/config";

pub mod structs;
use crate::commands::helpers::get_file_length;
use crate::commands::structs::HashObjectCreator;
use crate::commands::structs::ObjectType;
use crate::commands::structs::Head;
use crate::commands::structs::StagingArea;

use self::structs::ServerConnection;

mod helpers;

pub trait Command {
    fn execute(&self, head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>>;
}

pub struct Init;

impl Init {
    pub fn new() -> Self {
        Init {  }
    }
}

impl Command for Init {
    /// Executes the `git init` command, initializing a new Git repository in the current directory.
    /// This function initializes a new Git repository by creating the necessary directory structure
    /// for branches, tags, and objects. It also sets the default branch to 'main' and creates an empty
    ///  index file. If successful, it returns an empty string; otherwise, it returns an error message.
    fn execute(&self, head: &mut Head, _: Option<Vec<&str>>) -> Result<String, Box<dyn Error>>{
        
        let _refs_heads = fs::create_dir_all(R_HEADS);
        
        let _refs_tags = fs::create_dir(R_TAGS)?;
        let _obj = fs::create_dir(OBJECT)?;
        let _pack = fs::create_dir(PACK)?;

        let mut _config_file = fs::File::create(CONFIG_FILE)?;
        let mut head_file = fs::File::create(HEAD_FILE)?;
        head_file.write_all(b"ref: refs/heads/main")?;

        let _main = fs::File::create(".git/refs/heads/main")?; //esto no esta ideal hacerlo aca
        helpers::create_new_branch(DEFAULT_BRANCH_NAME, head)?;
        let _index_file = fs::File::create(INDEX_FILE)?;
        
        Ok(String::new())    
    }
}

pub struct Branch;

impl Branch {
    pub fn new() -> Self {
        Branch {  }
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
/// ```rust
/// let mut head = Head::new(); // Initialize a Head instance.
/// let args1 = Some(&["my-branch1"]); // Command-line arguments.
/// let result1 = Branch.execute(&mut head, args1);
/// assert!(result1.is_ok());
/// let args2 = Some(&["-d", "my-branch1", "-m", "my-branch2"]); // Command-line arguments.
/// let result2 = Branch.execute(&mut head, args2);
/// assert!(result2.is_ok());
/// ```
impl Command for Branch {
    fn execute(&self, head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
	    let list_branches_flag = args.is_none();
	    let mut delete_flag = false;
	    let mut rename_flag = false;
	    let mut first_branch_name: Option<String> = None;
	    let mut second_branch_name: Option<String> = None;
	    let arg_slice = args.unwrap_or(Vec::new());


	    for arg in arg_slice { // Note the & in for &arg
	        match arg {
	            DELETE_FLAG => delete_flag = true,
	            RENAME_FLAG => rename_flag = true,
	            _ => {
	            	if first_branch_name.is_none() {
	                    first_branch_name = Some(arg.to_string());
	                } else if second_branch_name.is_none() {
	                    second_branch_name = Some(arg.to_string());
	                }
	            },
	        }
	    }

	    /*
			- if there are no args, print list of branches
			- if there is "-d" flag, and a branch name, delete it
			- if there is "-m" flag, and there isn't "-d" flag, and 2 branch names, rename the "first branch name" to the "second branch name"
			- if there is no flags and a branch name, create a branch with that name
	    */
	    match (list_branches_flag, delete_flag, rename_flag, first_branch_name, second_branch_name) {
	        (true, _, _, _, _) => head.print_all(),
	        (_, true, _, Some(name), _) => head.delete_branch(&name)?,
	        (_, false, true, Some(old_name), Some(new_name)) => head.rename_branch(&old_name, &new_name)?,
	        (false, false, false, Some(name), _) => helpers::create_new_branch(&name, head)?,
	        _ => {}
	    }
	    Ok(String::new())
	}
}

pub struct Checkout;

impl Checkout {
    pub fn new() -> Self {
        Checkout {}
    }
}

impl Command for Checkout {
    /// Executes the `git checkout` command, which changes the current branch to the specified one.
    /// It updates the `HEAD` file to point to the new branch if it's different from the current branch.
    /// If successful, it returns an empty string; otherwise, it returns an error message.
    fn execute(&self, _head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        match args {
            Some(args) => {
    
                let branch_path = format!("{}/{}", R_HEADS, args[0]);
                if !fs::metadata(branch_path).is_ok() {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Branch name did not match any file known.",
                    )))
                }
                let new_head_content = format!("ref: refs/heads/{}", args[0]);
        
                let head_file_content = helpers::read_file_content(HEAD_FILE)?;
        
                if head_file_content == new_head_content {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Already on specified branch",
                    )))
                }
                
                let mut head_file = fs::File::create(HEAD_FILE)?;
                head_file.write_all(new_head_content.as_bytes())?;
            }
            None => return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "No branch name was provided",
            ))),
        }

        Ok(String::new())
    }
}

pub struct CatFile;

impl CatFile {
    pub fn new () -> Self {
        CatFile{}
    }
}

impl Command for CatFile {
    /// Executes the `cat-file` command, which displays information about a Git object's type or size.
    fn execute(&self, _head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        match args {
            Some(args) => {
                let path = format!(".git/objects/{}/{}", &args[1][..2], &args[1][2..]);
                let file = fs::File::open(path)?;

                let mut decoder = Decoder::new(file)?;

                let mut header = [0u8; 8];
                decoder.read_exact(&mut header)?;

                let header_str = str::from_utf8(&header)?;

                // Extract the object type and size
                let parts: Vec<&str> = header_str.trim_end().split(' ').collect();

                match args[0] {
                    TYPE_FLAG => println!("{}", parts[0]),
                    SIZE_FLAG => println!("{}", parts[1]),
                    _ => eprintln!(""),
                }
            }
            None => eprintln!("")
        }

        Ok(String::new())
    }
}

pub struct HashObject;

impl HashObject {
    pub fn new() -> Self {
        HashObject {  }
    }
}

impl Command for HashObject {
    /// Executes the `hash-object` command, which calculates the hash of a given file or data.
    /// If the write flag is specified, the object is created as a file in the objects subdirectory.
    /// Default object type is "blob" but can be specified with type flag.
    fn execute(&self, _head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        let arg_slice = args.unwrap_or(Vec::new());
        let mut path: &str = "";
        let mut obj_type = ObjectType::Blob;
        let mut write = false;
        for mut i in 0..arg_slice.len() {
            match arg_slice[i] {
                TYPE_FLAG => {
                    i += 1;
                    if let Some(new_obj_type) = ObjectType::new(arg_slice[i]) {
                        obj_type = new_obj_type;
                    } else {
                        eprintln!("Unknown object type for input: {}", arg_slice[i]);
                        return Ok(String::new());
                    }
                }
                WRITE_FLAG => write = true,
                _ => path = arg_slice[i],
            }
        }  
        if path.is_empty() {
            eprintln!("Please provide a file path or data to hash.");
            return Ok(String::new());
        }
        let content = helpers::read_file_content(path)?;
        if write {
            let file_len = helpers::get_file_length(path)?;
            return HashObjectCreator::write_object_file(content, obj_type, file_len);
        }
        else {
            println!("{}", helpers::generate_sha1_string(content.as_str()));
        }
        Ok(String::new())
    }
}

pub struct Commit {
    stg_area: StagingArea,
}

impl Commit {
    pub fn new() -> Self {
        Commit { stg_area: StagingArea::new() }
    }

    /// Generates the content for a new commit.    
    fn generate_commit_content(&self, tree_hash: String, message: Option<&str>, branch_path: &str) -> Result<String, Box<dyn Error>> {
        let head_commit = helpers::read_file_content(branch_path)?;
        let mut content = format!("{}\n{}", tree_hash, head_commit);
        if let Some(message) = message {
            content = format!("{}\n{}", content, message);
        }
        Ok(content)
    }

}

impl Command for Commit {
    /// Executes the `commit` command, creating a new commit for the changes in the staging area.
    /// To achieve this, it creates a "tree" which is the index file turned into a tree object.
    /// Then it creates a commit file, which contains the tree object hash, the commit's parent
    /// commits and the given message with the message flag.
    fn execute(&self, _head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        if helpers::get_file_length(INDEX_FILE)? == 0 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "No changes staged for commit",
            )))
        } 

        let mut message: Option<&str> = None;
        let mut message_flag = false;
        let arg_slice = args.unwrap_or(Vec::new());

	    for arg in arg_slice {
	        match arg { 
                MESSAGE_FLAG => message_flag = true,
                _ => message = Some(arg),
            }
        }
        let index_file_content = helpers::read_file_content(INDEX_FILE)?;
        let tree_hash = HashObjectCreator::write_object_file(index_file_content.clone(), ObjectType::Tree, index_file_content.as_bytes().len() as u64)?;
    
        let branch_path = helpers::get_current_branch_path()?;
        message = if message_flag { message } else { None };
        let commit_content = self.generate_commit_content(tree_hash, message, &branch_path)?;

        let commit_object_hash = HashObjectCreator::write_object_file(commit_content.clone(), ObjectType::Commit, commit_content.as_bytes().len() as u64)?;

        let mut branch_file = fs::File::create(branch_path)?;
        branch_file.write_all(commit_object_hash.as_bytes())?;

        self.stg_area.unstage_index_file()?;
        Ok(String::new())
    }
}

pub struct Rm {
    stg_area: StagingArea,
}

impl Rm {
    pub fn new() -> Self {
        Rm { stg_area: StagingArea::new() }
    }
}

impl Command for Rm {
    /// Receives a file path and removes it from the staging area.
    fn execute(&self, _head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        match args {
            Some(args) => {
                self.stg_area.remove_file(args[0])?;
            }
            None => return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Did not receive a file path to remove",
            ))),
        }
        Ok(String::new())
    }
}

pub struct Add {
    stg_area: StagingArea,
}

impl Add {
    pub fn new() -> Self {
        Add { stg_area: StagingArea::new() }
    }
}

impl Command for Add {
    /// Receives a file path and adds it to the staging area.
    fn execute(&self, head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        match args {
            Some(args) => {
                self.stg_area.add_file(head, args[0])?;
            }
            None => return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Did not receive a file path to add",
            ))),
        }
        Ok(String::new())
    }
}

pub struct Status;

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
    fn execute(&self, _head: &mut Head, _args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        let branch_path = helpers::get_current_branch_path()?;
        let last_commit_hash: String = helpers::read_file_content(&branch_path)?;
        let mut no_changes = true;
        let mut tree_objects: Vec<String> = Vec::new();
        if !last_commit_hash.is_empty() {
            let last_commit_path = format!("{}/{}/{}", OBJECT, &last_commit_hash[..2], &last_commit_hash[2..]);
            let decompressed_data = helpers::decompress_file_content(helpers::read_file_content_to_bytes(&last_commit_path)?)?;
            let commit_file_content: Vec<String> = decompressed_data.split('\0').map(String::from).collect();
            let commit_file_lines: Vec<String> = commit_file_content[1].lines().map(|s| s.to_string()).collect();

            let tree_hash = &commit_file_lines[0];
            let tree_object_path = format!("{}/{}/{}", OBJECT, &tree_hash[..2], &tree_hash[2..]);
            let tree_content = helpers::decompress_file_content(helpers::read_file_content_to_bytes(&tree_object_path)?)?;
            let tree_contents_split: Vec<String> = tree_content.split('\0').map(String::from).collect();
            tree_objects =  tree_contents_split[1].lines().map(|s| s.to_string()).collect();
        }
        
        let index_file_content = helpers::read_file_content(INDEX_FILE)?; 
        let index_objects: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();

        for pos in 0..(index_objects.len()) {
            let index_file_line: Vec<&str> = index_objects[pos].split(';').collect();
            if pos < tree_objects.len() {
                let tree_file_line: Vec<&str> = tree_objects[pos].split(';').collect();
                if tree_file_line[1] != index_file_line[1] && index_file_line[2] == "2" {
                    no_changes = false;
                    println!("modified: {} (Staged)", index_file_line[0]);
                    continue;
                }
                let current_object_content = helpers::read_file_content(index_file_line[0])?;
                let current_object_hash = HashObjectCreator::generate_object_hash(ObjectType::Blob, get_file_length(index_file_line[0])?, &current_object_content);
                if current_object_hash != tree_file_line[1] && index_file_line[2] == "0" {
                    no_changes = false;
                    println!("modified: {} (Unstaged)", index_file_line[0]);
                }
            }
            else {
                no_changes = false;
                println!("new file: {} (Staged)", index_file_line[0]);
            }
        }
        if no_changes {
            println!("nothing to commit, working tree clean");
        }
        Ok(String::new())
    }
}

pub struct Remote;

impl Remote {
    pub fn new() -> Self {
        Remote {}
    }

    /// Adds a new remote repository configuration to the Git configuration file.
    fn add_new_remote(&self, remote_name: String, url: String) -> Result<(), Box<dyn Error>> {
        let config_content = helpers::read_file_content(CONFIG_FILE)?;

        let section_header = format!("[remote '{}']", remote_name);
        let new_config_content = format!("{}{}\nurl = {}\n", config_content, section_header, url);

        if config_content.contains(&section_header) { //en git permite agregar mas de un remote con mismo nombre si su config o url son distintos, me parece que complejiza mucho y por ahora mejor no poder agregar dos de mismo nombre
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Remote already exists in the configuration.",
            )));
        }

        let mut config_file = fs::File::create(CONFIG_FILE)?;
        config_file.write_all(new_config_content.as_bytes())?;

        Ok(())
    }

    /// Removes a specified remote repository configuration from the Git configuration file.
    fn remove_remote(&self, remote_name: String) -> Result<(), Box<dyn Error>> {
        let config_content = helpers::read_file_content(CONFIG_FILE)?;

        let remote_header = format!("[remote '{}']", remote_name);
        let mut new_config_content = String::new();
        let mut is_inside_remote_section = false;

        for line in config_content.lines() {
            if line == remote_header {
                is_inside_remote_section = true;
            }
            else if line.starts_with("[") {
                is_inside_remote_section = false;
            }
            if !is_inside_remote_section {
                new_config_content.push_str(line);
                new_config_content.push('\n');
            }
        }

        let mut config_file = fs::File::create(CONFIG_FILE)?;
        config_file.write_all(new_config_content.as_bytes())?;

        Ok(())
    }

    /// Lists and prints the names of remote repositories configured in the Git configuration.
    fn list_remotes(&self) -> Result<(), Box<dyn Error>> {
        let config_content = helpers::read_file_content(CONFIG_FILE)?;

        for line in config_content.lines() {
            if line.starts_with("[remote '") {
                let remote_name = line
                .trim_start_matches("[remote '")
                .trim_end_matches("']");
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
    fn execute(&self, _head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        if args.is_none() {
            self.list_remotes()?;
            return Ok(String::new());
        }
        let mut add_flag = false;
        let mut remove_flag = false;
        let mut name = None;
        let mut url = None;
        let arg_slice = args.unwrap_or(Vec::new());

	    for arg in arg_slice {
	        match arg { 
                ADD_FLAG => add_flag = true,
                REMOVE_FLAG => remove_flag = true,
                _ => if name.is_none() {
                        name = Some(arg.to_string());
                    } else if url.is_none() {
                        url = Some(arg.to_string());
                    },
            }
        }
        match (add_flag, remove_flag, name, url) {
	        (true, _, Some(name), Some(url)) => self.add_new_remote(name, url)?,
	        (_, true, Some(name), _) => self.remove_remote(name)?,
	        _ => {}
	    }
        Ok(String::new())
    }
}

pub struct PackObjects;

impl PackObjects {
    pub fn new() -> Self {
        PackObjects {}
    }
}

impl Command for PackObjects {
    /// Execute the `PackObjects` command.
    /// This command generates a Git pack file that contains compressed Git objects.
    /// The pack file format is used to efficiently store objects and their history.
    /// It also creates an index file that helps locate objects in the pack file.
    fn execute(&self, _head: &mut Head, _args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {  
        // Open pack and index files
        let mut pack_file = fs::File::create(".git/pack/pack_file.pack")?;
        // Create an uncompressed pack file
        let mut pack_file_content = Vec::new();
        let mut index_entries: Vec<u8> = Vec::new();
    
        // List all objects in the .git/objects directory
        let mut objects_list = Vec::new();
        helpers::list_files_recursively(".git/objects", &mut objects_list)?;
        let mut object_count: u32 = 0;
        let mut offset: u64 = 12;
        // Iterate through objects
        for object_path in objects_list {
            object_count += 1;
            let decompressed_data: String = helpers::decompress_file_content(helpers::read_file_content_to_bytes(&object_path)?)?;
            let file_content: Vec<String> = decompressed_data.split('\0').map(String::from).collect();
            let object_header: Vec<String> = file_content[0].split(' ').map(String::from).collect();
            let object_type = ObjectType::new(&object_header[0]).unwrap_or(ObjectType::Blob);
            let object_size: u64 = object_header[1].parse()?;
            let object_content: &str = &file_content[1];
            println!("header: {:?}", object_header);
            println!("content: {}", object_content);
            // Calculate the SHA-1 hash of the object content
    
            index_entries.extend_from_slice(&object_type.get_object_for_pack_file().to_be_bytes());  // Object type
            index_entries.extend_from_slice(&(object_size as u32).to_be_bytes());  // Object size
            index_entries.extend_from_slice(helpers::generate_sha1_string(&decompressed_data).as_bytes());  // SHA-1 hash bytes

            // Calculate the offset in the pack file (you need to keep track of this as you write to the pack file).
            offset += object_size;

            index_entries.extend_from_slice(&offset.to_be_bytes());  // Offset in the pack file

            // Append object content to the uncompressed pack file
            let header_byte = ((object_type.get_object_for_pack_file() & 0x07) << 4) | ((object_size & 0x0F) as u8);
            pack_file_content.extend_from_slice(&[header_byte]);
            pack_file_content.extend_from_slice(&helpers::compress_content(&object_content)?);
        }
    
        // Sort the object information by SHA-1 hash
        // object_info_list.sort();
        let mut pack_file_final = Vec::new();
        // Generate pack header
        let version = [0u8, 0u8, 0u8, 2u8];
        pack_file_final.extend_from_slice(b"PACK");
        pack_file_final.extend_from_slice(&version);
        pack_file_final.extend_from_slice(&object_count.to_be_bytes());
        //pack_file.write_all(helpers::generate_sha1_string_from_bytes(&pack_header).as_bytes())?;

        pack_file_final.extend_from_slice(&pack_file_content);
        // Write the uncompressed pack file content to the pack file
        pack_file.write_all(&pack_file_final)?;
    
        // Calculate the SHA-1 hash of the entire pack file content
        let pack_checksum = helpers::generate_sha1_string_from_bytes(&pack_file_final);
        println!("pack checksum: {}", pack_checksum);
        pack_file.write_all(pack_checksum.as_bytes())?;

        // let mut index_file = fs::File::create(".git/pack/pack_file.idx")?;
        // let index_header = 

        // index_file.write_all(index_header);
        // index_file.write_all(fanout_table);
        // index_file.write_all(&index_entries);
        // index
    
        Ok(String::new())
    }
}

pub struct Push;

impl Push {
    /// Creates a new `Push` instance.
    pub fn new() -> Self {
        Push {}
    }
}


impl Command for Push {
    fn execute(&self, _head: &mut Head, _args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        //Pack and index files are created in .git/pack/ directory
        let pack_objects = PackObjects::new();
        pack_objects.execute(_head, None)?; 

        let mut server_connection = ServerConnection::new();
        server_connection.receive_pack()?;

        Ok(String::new())
    }
}

pub struct Clone;

impl Clone {
    /// Creates a new `Clone` instance.
    pub fn new() -> Self {
        Clone {}
    }
}

impl Command for Clone {
    fn execute(&self, _head: &mut Head, _args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        let server_connection = ServerConnection::new();
        server_connection.clone_from_remote()?;

        Ok(String::new())
    }
}

/// This module defines the `Log` struct, which is responsible for implementing the "git log" command.
/// It provides methods to generate log entries and execute the command.

pub struct Log;

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
    fn generate_log_entries(&self, entries: &mut Vec<String>, base_commit: String) -> Result<String, Box<dyn Error>> {
        if base_commit.len() < 4 {
            return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        "Error: Invalid Commit ID. It's too short",
                    )))
        }

        let current_commit = if base_commit == HEAD { helpers::get_head_commit()? } else { base_commit };

        if entries.contains(&current_commit) {
            // don't process it again
            return Ok(String::new());
        }

        //println!("starting to generate logs for {:?}", current_commit.clone());
        let commit_path = format!("{}/{}/{}", OBJECT, &current_commit[..2], &current_commit[2..]);
        //println!("going to {:?}", commit_path.clone());
        let decompressed_data = helpers::decompress_file_content(helpers::read_file_content_to_bytes(&commit_path)?)?;
        //println!("decompressed data {:?}", decompressed_data.clone());
        
        // trim header
        let commit_file_content: Vec<String> = decompressed_data.split('\0').map(String::from).collect();

        let commit_file_lines: Vec<String> = commit_file_content[1].lines().map(|s| s.to_string()).collect();
        
        let parent_commit_trimmed = &commit_file_lines[1];

        entries.push(current_commit);

        if parent_commit_trimmed.is_empty() {            
            //root commit
            //println!("returning, found root commit");
            return Ok(String::new());
        }

        //println!("parent commit {:?}", parent_commit_trimmed.clone());
        self.generate_log_entries(entries, parent_commit_trimmed.clone())?;
        Ok(String::new())
    }
}

impl Command for Log {
    /// Executes the "git log" command.
    ///
    /// # Arguments
    ///
    /// * `_head` - A mutable reference to the `Head` structure (not used in this implementation).
    /// * `args` - An optional slice of arguments passed to the command.
    ///
    /// # Returns
    ///
    /// A `Result` containing the execution result or an error message. 
    fn execute(&self, _head: &mut Head, args: Option<Vec<&str>>) -> Result<String, Box<dyn Error>> {
        // Extract the arguments from the provided slice or use an empty slice if none is provided
        let arg_slice = args.unwrap_or(Vec::new());

        // Initialize vectors to store log entries (included and excluded)        
        let mut log_entries = Vec::new();
        let mut log_entries_excluded = Vec::new();


        // Iterate through the provided arguments
        for arg in arg_slice { // Note the & in for &arg
            // Check the first character of each argument
            if let Some(first_char) = arg.chars().next() {
                match first_char {
                    EXCLUDE_LOG_ENTRY => {
                        // Generate log entries for exclusion and store them in the excluded entries vector
                        self.generate_log_entries(&mut log_entries_excluded, arg[1..].to_string())?;
                        //println!("exclude {:?}", log_entries_excluded);

                    },
                    _ => {
                        // Generate log entries for inclusion and store them in the included entries vector
                        self.generate_log_entries(&mut log_entries, arg.to_string())?;
                        //println!("include {:?}", log_entries);

                    }
                }
            }
        }
        /*println!("result {:?}", log_entries.iter()
            .filter(| entry | !log_entries_excluded.contains(entry))
            .cloned()
            .collect::<Vec<String>>());*/

        // Filter out log entries that are in the excluded entries vector
        log_entries = log_entries.iter()
            .filter(| entry | !log_entries_excluded.contains(entry))
            .cloned()
            .collect::<Vec<String>>();

        // Display the resulting log entries
        for entry in &log_entries {
            println!("{:?}", entry);
        }
        
        // Return a successful result (an empty string in this case)
        Ok(String::new())
    }
}
