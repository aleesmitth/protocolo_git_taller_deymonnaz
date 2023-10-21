use std::{fs, error::Error, io, io::Write, io::Read};
extern crate crypto;
extern crate libflate;

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use libflate::zlib::Encoder;
use std::fmt;

const TYPE_FLAG: &str = "-t";
const WRITE_FLAG: &str = "-w";
const OBJECT: &str = ".git/objects";
const R_HEADS: &str = ".git/refs/heads";
const HEAD_FILE: &str = ".git/HEAD";
const R_TAGS: &str = ".git/refs/tags";
const DEFAULT_BRANCH_NAME: &str = "main";

pub struct Init {

}

impl Init {
    pub fn new() -> Self {
        Init {  }
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
		self.branches.push(name.to_string());
	}

	pub fn print_all(&self) {
		for s in self.branches.iter() {
			println!("branch:{}", s);
		}
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
}

/// Creates a new SHA1 hash and returns it as a string
fn generate_sha1_string(string: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(string);
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


    let hash_obj = HashObject::new();
    if let Err(error) = hash_obj.execute(&mut head, Some(&["-w", "-t", "tree", "hola.txt"])){
        eprintln!("{}", error);
        return; 
    }
}