use std::{fs, error::Error, io, io::Write, io::Read, str::FromStr, io::BufRead};
extern crate crypto;
extern crate libflate;

use crypto::sha1::Sha1;
use crypto::digest::Digest;
use libflate::zlib::{Encoder, Decoder};
// use std::str;
use crate::commands::structs::Head;

const R_HEADS: &str = ".git/refs/heads";
const HEAD_FILE: &str = ".git/HEAD";
const DEFAULT_BRANCH_NAME: &str = "main";
const INDEX_FILE: &str = ".git/index";


pub fn get_current_branch_path() -> Result<String, Box<dyn Error>> {
    let head_file_content = read_file_content(HEAD_FILE)?;
    let split_head_content: Vec<&str> = head_file_content.split(" ").collect();
    if let Some(branch_path) = split_head_content.get(1) { 
        let full_branch_path = format!(".git/{}", branch_path);
        return Ok(full_branch_path);
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "Eror reading branch path",
    )))
}

/// Returns lenght of the a given file's content
pub fn get_file_length(path: &str) -> Result<u64, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

/// Give a file's path it reads it's lines and returns them as a String
pub fn read_file_content(path: &str) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Give a file's path it reads it's lines and returns them as a Vec<u8>
pub fn read_file_content_to_bytes(path: &str) -> Result<Vec<u8>, io::Error> {
    let mut file_content: Vec<u8> = Vec::new();
    let mut file = fs::File::open(path)?;
    file.read_to_end(&mut file_content)?;
    Ok(file_content)
}

/// Given a file's content it compresses it using an encoder from the libflate external crate and
/// returns a Vec<u8> containing the encoded content
// pub fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
//     let mut encoder = Encoder::new(Vec::new())?;
//     encoder.write_all(content.as_bytes())?;
//     encoder.finish().into_result()
// }

pub fn compress_content(content: &str) -> Result<Vec<u8>, io::Error> {
    // Crea un nuevo `Encoder` y un vector para almacenar los datos comprimidos.
    let mut encoder = Encoder::new(Vec::new())?;

    // Escribe el contenido (en bytes) en el `Encoder`.
    encoder.write_all(content.as_bytes())?;

    // Finaliza la compresión y obtiene el resultado comprimido.
    let compressed_data = encoder.finish().into_result()?;

    // Devuelve el resultado comprimido.
    Ok(compressed_data)
}

/// This function takes a `Vec<u8>` containing compressed data, decompresses it using
/// the zlib decoder, and returns the decompressed content as a `String`.
pub fn decompress_file_content(content: Vec<u8>) -> Result<String, io::Error> {
    let mut decompressed_data= String::new();
    
    let mut decoder = Decoder::new(&content[..])?;
    decoder.read_to_string(&mut decompressed_data)?;
    Ok(decompressed_data)
} 

pub fn generate_sha1_string(branch_name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(branch_name);
    hasher.result_str()
}

pub fn create_new_branch(branch_name: &str, head: &mut Head) -> Result<(), Box<dyn Error>> { 
    let branch_path = format!("{}/{}", R_HEADS, branch_name);

    let mut branch_file = fs::File::create(&branch_path)?;

    if branch_name == DEFAULT_BRANCH_NAME {
        write!(branch_file, "")?;
    }
    else {
        write!(branch_file, "{}", generate_sha1_string(branch_name))?; //aca necesito buscar el ultimo commit del branch anterior
    }
    head.add_branch(branch_name);

    Ok(())
}

pub fn update_file_with_hash(object_hash: &str, new_status: &str, file_path: &str) -> io::Result<()> {
    // Read the file into a vector of lines.
    let file_contents = fs::read_to_string(INDEX_FILE)?;

    // Split the file contents into lines.
    let mut lines: Vec<String> = file_contents.lines().map(|s| s.to_string()).collect();

    // Search for the hash in the lines.
    let mut found = false;
    for line in &mut lines {
        if line.starts_with(file_path) {
            found = true;
            // Replace the existing line with the hash and "1".
            *line = format!("{};{};{}", file_path, object_hash, new_status);
            break;
        }
    }

    // If the hash was not found, add a new line.
    if !found {
        lines.push(format!("{};{};{}", file_path, object_hash, new_status));
    }

    // Join the lines back into a single string.
    let updated_contents = lines.join("\n");

    // Write the updated contents back to the file.
    fs::write(INDEX_FILE, updated_contents)?;

    Ok(())
}

pub fn remove_object_from_file(file_path: &str) -> io::Result<()> {
    // Read the file into a vector of lines.
    
    let file_contents = fs::read_to_string(INDEX_FILE)?;
    
    // Split the file contents into lines.
    let mut lines: Vec<String> = file_contents.lines().map(|s| s.to_string()).collect();

    // Search for the hash in the lines.
    let mut found_index: Option<usize> = None;
    for (index, line) in lines.iter().enumerate() {
        
        if line.starts_with(file_path) {
            found_index = Some(index);
            break;
        }
    }

    // If the hash was found, remove the line.
    if let Some(index) = found_index {
        lines.remove(index);
    }
    else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "path did not match any files",
        ));
    }

    // Join the lines back into a single string.
    let updated_contents = lines.join("\n");

    // Write the updated contents back to the file.
    fs::write(INDEX_FILE, updated_contents)?;

    Ok(())
}