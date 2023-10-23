
use std::{fs, error::Error, io::Write, io::Read};

use std::{fs, error::Error, io, io::Write, io::Read};
extern crate crypto;
extern crate libflate;

use crypto::sha1::Sha1;
use crypto::digest::Digest;
use libflate::zlib::{Encoder, Decoder};
use std::str;

use libflate::zlib::Encoder;
use std::fmt;

const TYPE_FLAG: &str = "-t";
const WRITE_FLAG: &str = "-w";
const OBJECT: &str = ".git/objects";

const DELETE_FLAG: &str = "-d";
const RENAME_FLAG: &str = "-m";

const R_HEADS: &str = ".git/refs/heads";
const HEAD_FILE: &str = ".git/HEAD";
const R_TAGS: &str = ".git/refs/tags";
const DEFAULT_BRANCH_NAME: &str = "main";

const TYPE: &str = "-t";
const SIZE: &str = "-s";

pub struct Init {

}

impl Init {
    pub fn new() -> Self {
        Init {  }
    }
}
pub struct Branch {

}

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
    fn execute(&self, _head: &mut Head, args: Option<&[&str]>) -> Result<(), Box<dyn Error>> {
        match args {
            Some(args) => {

                //let first_arg = args[1..];
                let path = format!(".git/objects/{}/{}", &args[1][..2], &args[1][2..]);
                let file = fs::File::open(path)?;

                let mut decoder = Decoder::new(file)?;

                let mut header = [0u8; 8];
                decoder.read_exact(&mut header)?;
                //let header_str = str::from_utf8(&header)?;

                let header_str = str::from_utf8(&header)?;

                // Extract the object type and size
                let parts: Vec<&str> = header_str.trim_end().split(' ').collect();

                match args[0] {
                    TYPE => println!("{}", parts[0]),
                    SIZE => println!("{}", parts[1]),
                    _ => eprintln!(""),
                }
            }
            None => eprintln!("")
        }

        Ok(())
    }
}

impl Command for Init {
    fn execute(&self, head: &mut Head, _: Option<&[&str]>) -> Result<(), Box<dyn Error>>{

        let _refs_heads = fs::create_dir_all(R_HEADS);
        let _refs_tags = fs::create_dir(R_TAGS)?;
        let _obj = fs::create_dir(OBJECT)?;

        create_new_branch(DEFAULT_BRANCH_NAME, head)?;

        let mut head_file = fs::File::create(HEAD_FILE)?;
        head_file.write_all(b"ref: refs/heads/main")?;
        
        Ok(())    

        
    }
}


impl Command for HashObject {
    fn execute(&self, _head: &mut Head, args: Option<&[&str]>) -> Result<(), Box<dyn Error>> {
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
                        return Ok(());
                    }
                }
                WRITE_FLAG => write = true,
                _ => path = arg_slice[i],
            }
        }  
        if path.is_empty() {
            eprintln!("Please provide a file path or data to hash.");
            return Ok(());
        }
        let content = read_file_content(path)?;
        if write {
            write_object_file(content, obj_type, path)?;
        }
        else {
            println!("{}", generate_sha1_string(content.as_str()));
        }
        Ok(())
    }
}

fn write_object_file(content: String, obj_type: ObjectType, path: &str) -> Result<(), Box<dyn Error>> {
    let data = format!("{} {}\0{}", obj_type, get_file_length(path)?, content);
    let hashed_data = generate_sha1_string(data.as_str());
    let compressed_content = compress_content(content.as_str())?;

    let obj_directory_path = format!("{}/{}", OBJECT, &hashed_data[0..2]);
    fs::create_dir(&obj_directory_path)?;

    let object_file_path = format!("{}/{}", obj_directory_path, &hashed_data[2..]);
    let mut object_file = fs::File::create(object_file_path)?;
    write!(object_file, "{:?}", compressed_content)?;    

    Ok(())
}

/// Returns lenght of the a given file's content
fn get_file_length(path: &str) -> Result<u64, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

/// Give a  file's path it reads it's lines and returns them as a String
fn read_file_content(path: &str) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Given a file's content it compresses it using an encoder from the libflate external crate and
/// returns a Vec<u8> containing the encoded content
fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
    let mut encoder = Encoder::new(Vec::new())?;
    encoder.write_all(content.as_bytes())?;
    encoder.finish().into_result()
}

pub trait Command {
    fn execute(&self, head: &mut Head, _: Option<&[&str]>) -> Result<(), Box<dyn Error>>;
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
    fn execute(&self, head: &mut Head, args: Option<&[&str]>) -> Result<(), Box<dyn Error>> {
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
	    Ok(())
	}



}

pub trait Command {
    fn execute(&self, head: &mut Head, args: Option<&[&str]>) -> Result<(), Box<dyn Error>>;
}

fn generate_sha1_string(branch_name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(branch_name);
    hasher.result_str()
}

fn create_new_branch(branch_name: &str, head: &mut Head) -> Result<(), Box<dyn Error>> { 
    let branch_path = format!("{}/{}", R_HEADS, branch_name);

    let mut branch_file = fs::File::create(&branch_path)?;

    write!(branch_file, "{}", generate_sha1_string(branch_name))?;
    head.add_branch(branch_name);

    Ok(())
}

fn main() {
    let mut head = Head::new();
    // let init = Init::new();
    // if let Err(error) = init.execute(&mut head, None){
    //     eprintln!("{}", error);
    //     return; 
    // }
    head.print_all();

    let mut cat = CatFile::new();
    if let Err(error) = cat.execute(&mut head, Some(&["-t", "000142551ee3ec5d88c405cc048e1d5460795102"])){
        eprintln!("{}", error);
        return; 
    }
}

    let hash_obj = HashObject::new();
    if let Err(error) = hash_obj.execute(&mut head, Some(&["-w", "-t", "tree", "hola.txt"])){
      eprintln!("{}", error);
        return;

  }


	if let Err(error) = branch.execute(&mut head, Some(&["branch-name"])){
		eprintln!("{}", error);
        return;
	}

	if let Err(error) = branch.execute(&mut head, Some(&["-d", "branch-name"])){
		eprintln!("{}", error);
        return;
	}

	if let Err(error) = branch.execute(&mut head, Some(&["branch-name"])){
		eprintln!("{}", error);
        return;
	}

	if let Err(error) = branch.execute(&mut head, Some(&["-m", "branch-name", "new_branch_name"])){
		eprintln!("{}", error);
        return;
	}
    
	if let Err(error) = branch.execute(&mut head, None){
		eprintln!("{}", error);
        return;
	}



}

