use std::{fs, error::Error, io::Write};
const GIT: &str = ".git";
const OBJECT: &str = ".git/objects";
const REFS: &str = ".git/refs";
const R_HEADS: &str = ".git/refs/heads";
const HEAD: &str = ".git/HEAD";
const R_TAGS: &str = ".git/refs/tags";
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
        let _dir = fs::create_dir(GIT)?;
        let _refs = fs::create_dir(REFS)?;
        let _obj = fs::create_dir(OBJECT)?;
        let _refs_heads = fs::create_dir(R_HEADS)?;
        head.add_branch("main");
        let mut head_file = fs::File::create(HEAD)?;
        head_file.write_all(b"ref: refs/heads/main")?;
        let _refs_tags = fs::create_dir(R_TAGS)?;

        Ok(())
    
    }
}

pub trait Command {
    fn execute(&self, head: &mut Head) -> Result<(), Box<dyn Error>>;
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
