use std::{fs, error::Error, io, io::Write, io::Read};
extern crate crypto;
extern crate libflate;

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use libflate::zlib::Encoder;

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
    fn execute(&self, _head: &mut Head, args: Option<&[&str]>) -> Result<(), Box<dyn Error>>{
        match args {
            Some(args) => {
                let mut path: &str = "";
                let mut obj_type = "blob";
                let mut write = false;
                for &arg in args {
                    match arg {
                        TYPE_FLAG => obj_type = arg,
                        WRITE_FLAG => write = true,
                        _ => path = arg,
                    }
                }                

                if path.is_empty() {
                    println!("Please provide a file path or data to hash.");
                    return Ok(());
                }

                let content = read_file_content(path)?;
                if write {
                    let data = format!("{} {}\0{}", obj_type, get_file_length(path)?, content);
                    let hashed_data = generate_sha1_string(data.as_str());
                    let compressed_content = compress_content(content.as_str())?;

                    let object_file_path = format!("{}/{}/{}", R_HEADS, &hashed_data[0..1], &hashed_data[2..]);
                    let mut object_file = fs::File::create(object_file_path)?;
                    write!(object_file, "{:?}", compressed_content)?;
                }
                else {
                    let hashed_content = generate_sha1_string(content.as_str());
                    println!("{}", hashed_content);
                }
            }
            None => {
                eprintln!("Please provide a file path or data to hash.");
            }
        }
        Ok(())
    }
}

fn get_file_length(path: &str) -> Result<u64, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

fn read_file_content(path: &str) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
    let mut encoder = Encoder::new(Vec::new())?;
    encoder.write_all(content.as_bytes())?;
    encoder.finish().into_result()
}

pub trait Command {
    fn execute(&self, head: &mut Head, _: Option<&[&str]>) -> Result<(), Box<dyn Error>>;
}

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
    let init = Init::new();
    if let Err(error) = init.execute(&mut head, None){
        eprintln!("{}", error);
        return; 
    }
    head.print_all();

}
