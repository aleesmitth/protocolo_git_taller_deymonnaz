use std::{fs, error::Error, io::Write, io::Read};
extern crate crypto;
extern crate libflate;

use crypto::sha1::Sha1;
use crypto::digest::Digest;
use libflate::zlib::{Encoder, Decoder};
use std::str;

const GIT: &str = ".git";
const OBJECT: &str = ".git/objects";
const REFS: &str = ".git/refs";
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

pub struct Head {
	branches: Vec<String>
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
    fn execute(&self, head: &mut Head, args: Option<&[&str]>) -> Result<(), Box<dyn Error>> {
        let _refs_heads = fs::create_dir_all(R_HEADS);
        let _refs_tags = fs::create_dir(R_TAGS)?;
        let _obj = fs::create_dir(OBJECT)?;

        create_new_branch(DEFAULT_BRANCH_NAME, head)?;

        let mut head_file = fs::File::create(HEAD_FILE)?;
        head_file.write_all(b"ref: refs/heads/main")?;
        
        Ok(())    

        
    }
}

pub trait Command {
    fn execute(&self, _head: &mut Head, args: Option<&[&str]>) -> Result<(), Box<dyn Error>>;
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