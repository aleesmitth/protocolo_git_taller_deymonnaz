mod commands;
use crate::commands::Command;
use crate::commands::structs::Head;

use std::fs::{File, self};
use std::io::{Read, Write};
use libflate::zlib::{Encoder, Decoder};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Your data to be compressed
    // let data: Vec<u8> = vec![120, 156, 5, 192, 193, 17, 192, 32, 8, 4, 192, 127, 170, 57, 229, 68, 41, 71, 4, 242, 202, 203, 254, 103, 178, 179, 160, 70, 169, 37, 39, 117, 86, 53, 158, 46, 32, 44, 91, 85, 58, 143, 35, 198, 196, 126, 28, 99, 55, 196, 202, 50, 241, 72, 161, 50, 193, 101, 210, 76, 133, 84, 9, 61, 29, 238, 207, 151, 247, 238, 55, 127, 62, 241, 24, 168];
    // let mut encoder = Encoder::new(Vec::new())?;

    // // Escribe el contenido (en bytes) en el `Encoder`.
    // encoder.write_all(&data)?;

    // let compressed_data = encoder.finish().into_result()?;

    // // 2. Write the compressed data to a file
    // let mut file = File::create("compressed_data")?;
    // file.write_all(&compressed_data)?;

    // // 3. Read the compressed data from the file
    // let mut decode_file = File::open("compressed_data")?;
    // // let mut read_compressed_data = Vec::new();
    // // decode_file.read_to_end(&mut read_compressed_data)?;

    // // // 4. Use DEFLATE (ZLIB) to decompress the data
    // let mut decompressed_data: Vec<u8> = Vec::new();
    
    // let mut decoder = Decoder::new(decode_file)?;
    // decoder.read_to_end(&mut decompressed_data)?;

    // // Now 'decompressed_data' contains the original data
    // println!("{:?}", decompressed_data);

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

    // if let Err(error) = add.execute(&mut head, Some(&["b.txt"])) {
    //     println!("{}", error);
    //     // return;
    // }

    let mut commit = commands::Commit::new();
    if let Err(error) = commit.execute(&mut head, Some(&["-m", "message"])) {
        println!("{}", error);
        // return;
    }

    let mut status: commands::Status = commands::Status::new();
    if let Err(error) = status.execute(&mut head, None) {
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

