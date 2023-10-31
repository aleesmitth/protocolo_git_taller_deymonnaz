use std::{fs, error::Error, io, io::Write, io::Read};
extern crate crypto;
extern crate libflate;

use crypto::sha1::Sha1;
use crypto::digest::Digest;
use libflate::zlib::{Encoder, Decoder};
use std::str;
use std::fmt;

const OBJECT: &str = ".git/objects";

const DELETE_FLAG: &str = "-d";
const RENAME_FLAG: &str = "-m";
const TYPE_FLAG: &str = "-t";
const WRITE_FLAG: &str = "-w";
const SIZE_FLAG: &str = "-s";
const MESSAGE_FLAG: &str = "-m";
const ADD_FLAG: &str = "add";
const REMOVE_FLAG: &str = "remove";

const R_HEADS: &str = ".git/refs/heads";
const HEAD_FILE: &str = ".git/HEAD";
const R_TAGS: &str = ".git/refs/tags";
const DEFAULT_BRANCH_NAME: &str = "main";
const INDEX_FILE: &str = ".git/index";
const CONFIG_FILE: &str = ".git/config";

pub struct Init;

impl Init {
    pub fn new() -> Self {
        Init {  }
    }
}
pub struct Branch;

impl Branch {
    pub fn new() -> Self {
        Branch {  }
    }
}

pub struct Head {
	branches: Vec<String>
}

pub struct HashObject;

impl HashObject {
    pub fn new() -> Self {
        HashObject {  }
    }
}

enum ObjectType {
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

pub struct Add {
    stg_area: StagingArea,
}

impl Add {
    fn new() -> Self {
        Add { stg_area: StagingArea::new() }
    }
}

impl Command for Add {
    fn execute(&self, head: &mut Head, args: Option<&[&str]>) -> Result<String, Box<dyn Error>> {
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

pub struct Rm {
    stg_area: StagingArea,
}

impl Rm {
    fn new() -> Self {
        Rm { stg_area: StagingArea::new() }
    }
}

impl Command for Rm {
    fn execute(&self, _head: &mut Head, args: Option<&[&str]>) -> Result<String, Box<dyn Error>> {
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

pub struct Commit {
    stg_area: StagingArea,
}

impl Commit {
    fn new() -> Self {
        Commit { stg_area: StagingArea::new() }
    }

    fn generate_commit_content(&self, tree_hash: String, message: Option<&str>, branch_path: &str) -> Result<String, Box<dyn Error>> {
        let head_commit = read_file_content(branch_path)?;
        println!("commit");
        //let mut content = String::new();
        // content.push_str(&tree_hash);
        // content.push_str(&head_commit);
        let mut content = format!("{}\n{}", tree_hash, head_commit);
        if let Some(message) = message {
            // content.push_str(message);
            content = format!("{}\n{}", content, message);
        }
        Ok(content)
    }

}

impl Command for Commit {
    fn execute(&self, head: &mut Head, args: Option<&[&str]>) -> Result<String, Box<dyn Error>> {
        if get_file_length(INDEX_FILE)? == 0 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "No changes staged for commit",
            )))
        } 

        let mut message: Option<&str> = None;
        let mut message_flag = false;
        let arg_slice = args.unwrap_or(&[]);

	    for &arg in arg_slice { // Note the & in for &arg
	        match arg { 
                MESSAGE_FLAG => message_flag = true,
                _ => message = Some(arg),
            }
        }
        let hash_obj = HashObject::new();
        let tree_hash = hash_obj.execute(head, Some(&[WRITE_FLAG, TYPE_FLAG, "tree", INDEX_FILE]))?;
    
        let branch_path = get_current_branch_path()?;
        message = if message_flag { message } else { None };
        let commit_content = self.generate_commit_content(tree_hash, message, &branch_path)?;

        let commit_object_hash = HashObjectCreator::write_object_file(commit_content.clone(), ObjectType::Commit, commit_content.as_bytes().len() as u64)?;

        let mut branch_file = fs::File::create(branch_path)?;
        branch_file.write_all(commit_object_hash.as_bytes())?;

        self.stg_area.clear_index_file()?;
        Ok(String::new())
    }
}

fn get_current_branch_path() -> Result<String, Box<dyn Error>> {
    let head_file_content = read_file_content(HEAD_FILE)?;
    let split_head_content: Vec<&str> = head_file_content.split(" ").collect();
    if let Some(branch_path) = split_head_content.get(1) { 
        let full_branch_path = format!(".git/{}", branch_path);
        return Ok(full_branch_path);
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "Eror reading branch path",
    )))
}

pub struct Status;

impl Status {
    fn new() -> Self {
        Status {}
    }
}

impl Command for Status {
    fn execute(&self, _head: &mut Head, _args: Option<&[&str]>) -> Result<String, Box<dyn Error>> {
        let branch_path = get_current_branch_path()?;
        let last_commit_hash: String = read_file_content(&branch_path)?;
        let last_commit_path = format!("{}/{}/{}", OBJECT, &last_commit_hash[..2], &last_commit_hash[2..]);

        println!("Last Commit Path: {}", last_commit_path);

        let mut commit_file = fs::File::open(&last_commit_path)?;

        // Lee el contenido del archivo en un búfer
        // let mut buf = Vec::new();
        // commit_file.read_to_end(&mut buf)?;

        // println!("{:?}", buf);
        let buf = [120, 156, 5, 192, 177, 17, 192, 32, 12, 3, 192, 158, 105, 68, 236, 224, 48, 14, 56, 18, 21, 21, 251, 223, 241, 33, 180, 238, 166, 207, 146, 45, 164, 234, 249, 24, 28, 157, 85, 226, 244, 156, 248, 223, 192, 40, 101, 243, 156, 177, 120, 1, 78, 167, 14, 28];

        // Utiliza Decoder para descomprimir el contenido del búfer
        let mut decoder = Decoder::new(&buf[..])?;
        let mut last_commit_content_bytes = Vec::new();
        println!("{:?}", last_commit_content_bytes);

        if decoder.read_to_end(&mut last_commit_content_bytes).is_err() {
            return Err("Failed to read commit content".into());
        }

        let last_commit_content = String::from_utf8(last_commit_content_bytes)?;

        println!("Last Commit Content: {:?}", last_commit_content);

        Ok(String::new())
    }
}

pub struct Remote;

impl Remote {
    fn new() -> Self {
        Remote {}
    }
    /// Adds a new remote repository configuration to the Git configuration file.
    fn add_new_remote(&self, remote_name: String, url: String) -> Result<(), Box<dyn Error>> {
        let config_content = read_file_content(CONFIG_FILE)?;

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
        let config_content = read_file_content(CONFIG_FILE)?;

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
        let config_content = read_file_content(CONFIG_FILE)?;

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
    fn execute(&self, _head: &mut Head, args: Option<&[&str]>) -> Result<String, Box<dyn Error>> {
        if args.is_none() {
            self.list_remotes()?;
            return Ok(String::new());
        }
        let mut add_flag = false;
        let mut remove_flag = false;
        let mut name = None;
        let mut url = None;
        let arg_slice = args.unwrap_or(&[]);

	    for &arg in arg_slice {
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

impl Head {	
	pub fn new() -> Self {
		Head { branches: Vec::new() }
	}

	pub fn add_branch(&mut self, name: &str) {
		// Check if the branch name is not already in the vector
	    if !self.branches.iter().any(| branch | branch == name) {
	        self.branches.push(name.to_string());
	    }
	}

	pub fn delete_branch(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
	    // Use the retain method to remove branches with the specified name
	    self.branches.retain( | branch | branch != name);

	    Ok(())
	}

	pub fn rename_branch(&mut self, old_name: &str, new_name: &str) -> Result<(), Box<dyn Error>> {
	    // Find the branch with the old name and rename it to the new name
	    if let Some(branch) = self.branches.iter_mut().find( | branch | *branch == old_name) {
	        *branch = new_name.to_string();
	    }

	    Ok(())
	}

	pub fn print_all(&self) {
		for s in self.branches.iter() {
			println!("branch:{}", s);
		}
	}
}

pub struct CatFile;

impl CatFile {
    pub fn new () -> Self {
        CatFile{}
    }
}

impl Command for CatFile {
    fn execute(&self, _head: &mut Head, args: Option<&[&str]>) -> Result<String, Box<dyn Error>> {
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

impl Command for Init {
    fn execute(&self, head: &mut Head, _: Option<&[&str]>) -> Result<String, Box<dyn Error>>{

        let _refs_heads = fs::create_dir_all(R_HEADS);
        let _refs_tags = fs::create_dir(R_TAGS)?;
        let _obj = fs::create_dir(OBJECT)?;

        create_new_branch(DEFAULT_BRANCH_NAME, head)?;

        let mut head_file = fs::File::create(HEAD_FILE)?;
        head_file.write_all(b"ref: refs/heads/main")?;

        let _config_file = fs::File::create(CONFIG_FILE)?;
        let _index_file = fs::File::create(INDEX_FILE)?;
        
        Ok(String::new())    
    }
}


impl Command for HashObject {
    fn execute(&self, _head: &mut Head, args: Option<&[&str]>) -> Result<String, Box<dyn Error>> {
        let arg_slice = args.unwrap_or(&[]);
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
        let content = read_file_content(path)?;
        if write {
            let file_len = get_file_length(path)?;
            return HashObjectCreator::write_object_file(content, obj_type, file_len);
        }
        else {
            println!("{}", generate_sha1_string(content.as_str()));
        }
        Ok(String::new())
    }
}

pub struct HashObjectCreator;

impl HashObjectCreator {
    fn new() -> Self {
        HashObjectCreator {}
    }

    fn write_object_file(content: String, obj_type: ObjectType, file_len: u64) -> Result<String, Box<dyn Error>> {
        let data = format!("{} {}\0{}", obj_type, file_len, content);
        let hashed_data = generate_sha1_string(data.as_str());
        let compressed_content = compress_content(content.as_str())?;
        println!("{:?}", compressed_content);
        let obj_directory_path = format!("{}/{}", OBJECT, &hashed_data[0..2]);
        let _ = fs::create_dir(&obj_directory_path);
    
        let object_file_path = format!("{}/{}", obj_directory_path, &hashed_data[2..]);
        if fs::metadata(object_file_path.clone()).is_ok() {
            return Ok(hashed_data)
        }
        
        let mut object_file = fs::File::create(object_file_path.clone())?;
        write!(object_file, "{:?}", compressed_content)?;    
    
        Ok(hashed_data)
    }
}



/// Returns lenght of the a given file's content
fn get_file_length(path: &str) -> Result<u64, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

/// Give a file's path it reads it's lines and returns them as a String
fn read_file_content(path: &str) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Given a file's content it compresses it using an encoder from the libflate external crate and
/// returns a Vec<u8> containing the encoded content
// fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
//     let mut encoder = Encoder::new(Vec::new())?;
//     encoder.write_all(content.as_bytes())?;
//     encoder.finish().into_result()
// }

fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
    // Crea un nuevo `Encoder` y un vector para almacenar los datos comprimidos.
    let mut encoder = Encoder::new(Vec::new())?;

    // Escribe el contenido (en bytes) en el `Encoder`.
    encoder.write_all(content.as_bytes())?;

    // Finaliza la compresión y obtiene el resultado comprimido.
    let compressed_data = encoder.finish().into_result()?;

    // Devuelve el resultado comprimido.
    Ok(compressed_data)
}

fn decompress_data(compressed_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut decoder = Decoder::new(compressed_data)?;
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
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
    fn execute(&self, head: &mut Head, args: Option<&[&str]>) -> Result<String, Box<dyn Error>> {
	    let list_branches_flag = args.is_none();
	    let mut delete_flag = false;
	    let mut rename_flag = false;
	    let mut first_branch_name: Option<String> = None;
	    let mut second_branch_name: Option<String> = None;
	    let arg_slice = args.unwrap_or(&[]);


	    for &arg in arg_slice { // Note the & in for &arg
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
	        (false, false, false, Some(name), _) => create_new_branch(&name, head)?,
	        _ => {}
	    }
	    Ok(String::new())
	}



}

pub trait Command {
    fn execute(&self, head: &mut Head, args: Option<&[&str]>) -> Result<String, Box<dyn Error>>;
}

fn generate_sha1_string(branch_name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(branch_name);
    hasher.result_str()
}

fn create_new_branch(branch_name: &str, head: &mut Head) -> Result<(), Box<dyn Error>> { 
    let branch_path = format!("{}/{}", R_HEADS, branch_name);

    let mut branch_file = fs::File::create(&branch_path)?;

    if branch_name == DEFAULT_BRANCH_NAME {
        write!(branch_file, "")?;
    }
    else {
        write!(branch_file, "{}", generate_sha1_string(branch_name))?; //aca necesito buscar el ultimo commit del branch anterior
    }
    head.add_branch(branch_name);

    Ok(())
}


fn update_file_with_hash(object_hash: &str, new_status: &str, file_path: &str) -> io::Result<()> {
    // Read the file into a vector of lines.
    let file_contents = fs::read_to_string(INDEX_FILE)?;

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
    fs::write(INDEX_FILE, updated_contents)?;

    Ok(())
}

fn remove_object_from_file(file_path: &str) -> io::Result<()> {
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
    }
    else {
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

#[derive(Debug, PartialEq, Eq, Clone)]
enum FileStatus {
    Untracked,
    Modified,
    Staged,
}

#[derive(Debug)]
struct StagingArea;

impl StagingArea {
    fn new() -> Self {
        StagingArea {}
    }

    fn add_file(&self, head: &mut Head, path: &str) -> Result<(), Box<dyn Error>> {
        let hash_object = HashObject::new();
        let object_hash = hash_object.execute(head, Some(&["-w", path]))?;

        update_file_with_hash(&object_hash.as_str(), "2", path)?;

        Ok(())
    }

    fn remove_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        remove_object_from_file(path)?;
        Ok(())
    }

    fn clear_index_file(&self) -> Result<(), Box<dyn Error>> {
        fs::File::create(INDEX_FILE)?;
        Ok(())
    }

    // fn unstage_file(&mut self, path: &str) {
    //     if let Some(status) = self.files.get_mut(path) {
    //         *status = FileStatus::Modified;
    //     }
    // }

    // fn list_stagedfiles(&self) -> Vec<&str> {
    //     self.files
    //         .iter()
    //         .filter(|&(, status)|status == FileStatus::Staged)
    //         .map(|(path, _)| path.as_str())
    //         .collect()
    // }
}

fn main() {
    let mut head = Head::new();
    // let init = Init::new();
    // if let Err(error) = init.execute(&mut head, None){
    //     eprintln!("{}", error);
    //     return; 
    // }
    head.print_all();

    let remote = Remote::new();
    // if let Err(error) = remote.execute(&mut head, Some(&["add", "hola", "hola.com"])) {
    //     println!("{}", error);
    //     return;
    // }
    // if let Err(error) = remote.execute(&mut head, Some(&["add", "chau", "chau.com"])) {
    //     println!("{}", error);
    //     return;
    // }
    if let Err(error) = remote.execute(&mut head, None) {
        println!("{}", error);
        return;
    }
    if let Err(error) = remote.execute(&mut head, Some(&["remove", "hola"])) {
        println!("{}", error);
        return;
    }
    if let Err(error) = remote.execute(&mut head, None) {
        println!("{}", error);
        return;
    }
    // let add = Add::new();
    // if let Err(error) = add.execute(&mut head, Some(&["a/a.txt"])) {
    //     println!("{}", error);
    //     return;
    // }

    // let mut commit = Commit::new();
    // if let Err(error) = commit.execute(&mut head, Some(&["-m", "message"])) {
    //     println!("{}", error);
    //     return;
    // }

    // let mut status = Status::new();
    // if let Err(error) = status.execute(&mut head, None) {
    //     println!("{}", error);
    //     return;
    // }

    // let mut cat = CatFile::new();
    // if let Err(error) = cat.execute(&mut head, Some(&["-t", "000142551ee3ec5d88c405cc048e1d5460795102"])){
    //     eprintln!("{}", error);
    //     return; 
    // }

    // let hash_obj = HashObject::new();
    // if let Err(error) = hash_obj.execute(&mut head, Some(&["-w", "-t", "tree", "hola.txt"])){
    //     eprintln!("{}", error);
    //     return;
    // }

	// if let Err(error) = branch.execute(&mut head, Some(&["branch-name"])){
	// 	eprintln!("{}", error);
    //     return;
	// }

	// if let Err(error) = branch.execute(&mut head, Some(&["-d", "branch-name"])){
	// 	eprintln!("{}", error);
    //     return;
	// }

	// if let Err(error) = branch.execute(&mut head, Some(&["branch-name"])){
	// 	eprintln!("{}", error);
    //     return;
	// }

	// if let Err(error) = branch.execute(&mut head, Some(&["-m", "branch-name", "new_branch_name"])){
	// 	eprintln!("{}", error);
    //     return;
	// }
    
	// if let Err(error) = branch.execute(&mut head, None){
	// 	eprintln!("{}", error);
    //     return;
	// }
}

