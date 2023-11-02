mod commands;

use commands::Checkout;
use crate::commands::structs::Head;
use crate::commands::Command;

fn main() {
    let mut head = Head::new();
    // let init = commands::Init::new();
    // if let Err(error) = init.execute(&mut head, None){
    //     eprintln!("{}", error);
    //     return; 
    // }
    // head.print_all();

    // let branch = commands::Branch::new();
    // if let Err(error) = branch.execute(&mut head, Some(&["new"])) {
    //     println!("{}", error);
    //     return;
    // }

    let checkout = Checkout::new();
    if let Err(error) = checkout.execute(&mut head, Some(&["new"])) {
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

