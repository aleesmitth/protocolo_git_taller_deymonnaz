mod commands;
use commands::CatFile;


use crate::commands::structs::Head;
use crate::commands::Command;
use std::fmt::format;
use std::{fs, sync::mpsc, thread, io::BufRead, io::BufReader, io::Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {


    let mut head = Head::new();
    let init = commands::Init::new();
    if let Err(error) = init.execute(&mut head, None){
        eprintln!("{}", error);
        //return; 
    }
    let mut hash = String::new();
    let hash_object = commands::HashObject::new();
    match hash_object.execute(&mut head, Some(&["-w", "b.txt"])) {
        Ok(hashed) => hash = hashed,
        Err(error) => eprintln!("{}", error),
    }

    let mut stream = TcpStream::connect("127.0.0.1:9418")?;

    // Send a Git command (e.g., git-upload-pack).
    println!("aca");
    // let git_command = "git-upload-pack /path/to/repo.git";
    // let message_length = git_command.len() + 4; // Add 4 bytes for the length prefix
    // let message_length_bytes: [u8; 4] = (message_length as u32).to_be_bytes();
    // stream.write_all(&message_length_bytes)?;
    // stream.write_all(git_command.as_bytes())?;
    // let request = format!("{:04x}git-receive-pack /.git\0", "git-receive-pack /.git\0".len() + 4);
    // stream.write_all(request.as_bytes())?;
    // stream.flush()?;
    
    // // aca hay que hacer negotiation
    // let negotiation = format!("{:04x}want b10000000000000000000000000000000000000000\n", "want b10000000000000000000000000000000000000000\n".len() +4 );
    // stream.write_all(negotiation.as_bytes())?;
    // stream.write_all("0000".as_bytes())?;
    // stream.write_all("done\n".as_bytes())?;
    // stream.flush()?;

    let request = format!("{:04x}git-upload-pack /.git\0host=127.0.0.1\0", "git-upload-pack /.git\0host=127.0.0.1\0".len() + 4);
    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    let negotiation = format!("{:04x}want f20000000000000000000000000000000000000000 multi_ack side-band-64k ofs-delta\n", "f20000000000000000000000000000000000000000 multi_ack side-band-64k ofs-delta\n".len() +7 );
    println!("{}", negotiation);
    stream.write_all(negotiation.as_bytes())?;
    stream.write_all("0000".as_bytes())?;
    stream.write_all("done\n".as_bytes())?;
    stream.flush()?;

    let pack_objects = commands::PackObjects::new();
    if let Err(error) = pack_objects.execute(&mut head, None) {
        println!("{}", error);
        // return;
    }
    let pack_file_path = ".git/pack/pack_file.pack"; // Replace with your pack file path
    let mut pack_file = fs::File::open(pack_file_path)?;

    // Send the pack file to the server
    std::io::copy(&mut pack_file, &mut stream)?;

    // Read and process the server's response.
    let reader = BufReader::new(&stream);
    println!("despues");
    for line in reader.lines() {
        let line = line?;
        println!("ok: {}", line);
    }
    // // head.print_all();
    // let add = commands::Add::new();
    // if let Err(error) = add.execute(&mut head, Some(&["a/a.txt"])) {
    //     println!("{}", error);
    //     // return;
    // }

    // if let Err(error) = add.execute(&mut head, Some(&["b.txt"])) {
    //     println!("{}", error);
    //     // return;
    // }

    // let mut commit = commands::Commit::new();
    // if let Err(error) = commit.execute(&mut head, Some(&["-m", "message"])) {
    //     println!("{}", error);
    //     // return;
    // }

    // if let Err(error) = add.execute(&mut head, Some(&["b.txt"])) {
    //     println!("{}", error);
    //     // return;
    // }

    // let mut status: commands::Status = commands::Status::new();
    // if let Err(error) = status.execute(&mut head, None) {
    //     println!("{}", error);
    //     // return;
    // }

    // // let cat_file = commands::CatFile::new();
    // // if let Err(error) = cat_file.execute(&mut head, Some(&["-t", "b8b4a4e2a5db3ebed5f5e02beb3e2d27bca9fc9a"])) {
    // //     println!("{}", error);
    // //     // return;
    // // }

    // let mut remote = commands::Remote::new();
    // if let Err(error) = remote.execute(&mut head, Some(&["add", "origin", "127.0.0.1:9418"])) {
    //     println!("{}", error);
    //     // return;
    // }


    // let mut push = commands::Push::new();
    // if let Err(error) = push.execute(&mut head, None) {
    //     println!("{}", error);
    //     // return;
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

    fs::remove_dir_all(".git")?;

    Ok(())
}

