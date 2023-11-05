mod commands;
use commands::CatFile;

use crate::commands::structs::Head;
use crate::commands::Command;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut head = Head::new();
    let init = commands::Init::new();
    if let Err(error) = init.execute(&mut head, None){
        eprintln!("{}", error);
        //return; 
    }
    // head.print_all();
    let add = commands::Add::new();
    if let Err(error) = add.execute(&mut head, Some(&["a/a.txt"])) {
        println!("{}", error);
        // return;
    }

    if let Err(error) = add.execute(&mut head, Some(&["b.txt"])) {
        println!("{}", error);
        // return;
    }

    let mut commit = commands::Commit::new();
    if let Err(error) = commit.execute(&mut head, Some(&["-m", "message"])) {
        println!("{}", error);
        // return;
    }

    if let Err(error) = add.execute(&mut head, Some(&["b.txt"])) {
        println!("{}", error);
        // return;
    }

    let mut status: commands::Status = commands::Status::new();
    if let Err(error) = status.execute(&mut head, None) {
        println!("{}", error);
        // return;
    }

    // let cat_file = commands::CatFile::new();
    // if let Err(error) = cat_file.execute(&mut head, Some(&["-t", "b8b4a4e2a5db3ebed5f5e02beb3e2d27bca9fc9a"])) {
    //     println!("{}", error);
    //     // return;
    // }

    let mut pack_object: commands::PackObjects = commands::PackObjects::new();
    if let Err(error) = pack_object.execute(&mut head, None) {
        println!("{}", error);
        // return;
    }


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

    fs::remove_dir_all(".git")?;

    Ok(())
}

