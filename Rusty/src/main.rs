mod commands;

use crate::commands::structs::Head;
use crate::commands::Command;
use std::{fs, sync::mpsc, thread, io::BufRead, io::BufReader, io::Write};
use std::net::TcpStream;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {


    let mut head = Head::new();
    let init = commands::Init::new();
    if let Err(error) = init.execute(&mut head, None){
        eprintln!("{}", error);
        //return; 
    }
    // let mut hash = String::new();
    // let hash_object = commands::HashObject::new();
    // match hash_object.execute(&mut head, Some(&["-w", "b.txt"])) {
    //     Ok(hashed) => hash = hashed,
    //     Err(error) => eprintln!("{}", error),
    // }
    // println!("{}", hash);

    // let mut stream = TcpStream::connect("127.0.0.1:9418")?;

    // let request = format!("{:04x}git-upload-pack /.git\0host=127.0.0.1\0", "git-upload-pack /.git\0host=127.0.0.1\0".len() + 4);
    // stream.write_all(request.as_bytes())?;
    // stream.flush()?;

    // let pack_objects = commands::PackObjects::new();
    // if let Err(error) = pack_objects.execute(&mut head, None) {
    //     println!("{}", error);
    //     // return;
    // }
    // // let pack_file_path = ".git/pack/pack_file.pack"; // Replace with your pack file path
    // // let mut pack_file = fs::File::open(pack_file_path)?;

    // // // Send the pack file to the server
    // // std::io::copy(&mut pack_file, &mut stream)?;

    // // Read and process the server's response.
    // let reader = BufReader::new(&stream);
    // println!("despues");
    // for line in reader.lines() {
    //     let line = line?;
    //     println!("ok: {}", line);
    //     break;
    // }

    // let negotiation = format!("{:04x}want e12e849f30767b19e9b2f0ee625b820c56fd647d multi_ack side-band-64k\n", "want 00f2000000000000000000000000000000000000 multi_ack side-band-64k\n".len() + 4);
    // println!("{}", negotiation);
    // stream.write_all(negotiation.as_bytes())?;
    // stream.write_all(b"0000")?;
    // let done = format!("{:04x}done\n", "done\n".len()+4);
    // println!("{}", done);
    // stream.write_all(done.as_bytes())?;
    // stream.flush()?;

    // let mut response = String::new();
    // let mut buffer = [0; 4]; // Read the response's length
    
    // // Read the length of the server's response
    // stream.read_exact(&mut buffer)?;
    // let length = u32::from_be_bytes(buffer) as usize;

    // // Read the server's response
    // let mut response_buffer = vec![0u8; length];
    // stream.read_exact(&mut response_buffer)?;

    // // Convert the response to a string
    // response = String::from_utf8_lossy(&response_buffer).to_string();

    // if response.starts_with("ACK") {
    //     // Handle the ACK response
    //     println!("Request accepted!");
    // } else {
    //     // Handle other responses or errors
    //     println!("Request failed: {}", response);
    // }

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


    let mut push = commands::Push::new();
    if let Err(error) = push.execute(&mut head, None) {
        println!("{}", error);
        // return;
    }

    // let mut clone = commands::Clone::new();
    // if let Err(error) = clone.execute(&mut head, None) {
    //     println!("{}", error);
    //     // return;
    // }
    fs::remove_dir_all(".git")?;

    Ok(())
}

