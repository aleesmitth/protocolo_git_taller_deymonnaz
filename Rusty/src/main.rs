use std::{fs, error::Error, io::Write};
extern crate crypto;
use crypto::sha1::Sha1;
use crypto::digest::Digest;

//const GIT: &str = ".git";
const OBJECT: &str = ".git/objects";
//const REFS: &str = ".git/refs";
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

impl Command for Init {
    fn execute(&self, head: &mut Head, _: Option<&[&str]>) -> Result<(), Box<dyn Error>>{
        // usando create_dir_all() se puede evitar crear a todos uno por uno
        // asi que podemos borrar algunas de las ctes
        // tambien usando format!() se podria ir uniendo los paths 
        let _refs_heads = fs::create_dir_all(R_HEADS);
        let _refs_tags = fs::create_dir(R_TAGS)?;
        let _obj = fs::create_dir(OBJECT)?;

        //let mut head = Head::new();
        create_new_branch(DEFAULT_BRANCH_NAME, head)?;

        let mut head_file = fs::File::create(HEAD)?;
        head_file.write_all(b"ref: refs/heads/main")?;
        
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
	            "-d" => delete_flag = true,
	            "-m" => rename_flag = true,
	            _ => {
	            	if first_branch_name.is_none() {
	                    first_branch_name = Some(arg.to_string());
	                } else if second_branch_name.is_none() {
	                    second_branch_name = Some(arg.to_string());
	                }
	            },
	        }
	    }

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
    let init = Init::new();
    let branch = Branch::new();
    if let Err(error) = init.execute(&mut head, None){
        eprintln!("{}", error);
        return;
    }
    //head.print_all();

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
