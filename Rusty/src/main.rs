use std::{fs, error::Error, io::Write};
extern crate crypto;
use crypto::sha1::Sha1;
use crypto::digest::Digest;

const GIT: &str = ".git";
const OBJECT: &str = ".git/objects";
const REFS: &str = ".git/refs";
const R_HEADS: &str = ".git/refs/heads";
const HEAD: &str = ".git/HEAD";
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
    fn execute(&self, head: &mut Head) -> Result<(), Box<dyn Error>>{
        // usando create_dir_all() se puede evitar crear a todos uno por uno
        // asi que podemos borrar algunas de las ctes
        // tambien usando format!() se podria ir uniendo los paths 
        let _refs_heads = fs::create_dir_all(R_HEADS);
        let _refs_tags = fs::create_dir(R_TAGS)?;
        let _obj = fs::create_dir(OBJECT)?;

        let mut head = Head::new();
        create_new_branch(DEFAULT_BRANCH_NAME, &mut head)?;

        let mut head_file = fs::File::create(HEAD)?;
        head_file.write_all(b"ref: refs/heads/main")?;
        
        Ok(())    

        
    }
}

pub trait Command {
    fn execute(&self, head: &mut Head) -> Result<(), Box<dyn Error>>;
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
    let init = Init::new();
    if let Err(error) = init.execute(&mut head){
        eprintln!("{}", error);
        return; 
    }
    head.print_all();
}