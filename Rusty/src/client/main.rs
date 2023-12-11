use rusty::commands::commands::Command;
use rusty::commands::commands;
use rusty::commands::helpers;
use std::{io, env, fs};

/// This function takes a slice of strings and converts it into a vector of string slices.
/// Returns an `Option` containing a `Vec` of string slices if `args` is not empty. Returns `None` if `args` is empty.
fn parse_arguments(args: &[String]) -> Option<Vec<&str>> {
    if args.is_empty() {
        return None;
    }
    let arg_slices: Vec<&str> = args.iter().map(String::as_str).collect();
    Some(arg_slices)
}

// fn hex_to_bytes(hex_string: &str) -> Vec<u8> {
//     // Check if the length of the input string is even
//     if hex_string.len() % 2 != 0 {
//         panic!("Hexadecimal string must have an even number of characters");
//     }

//     let mut bytes = Vec::new();

//     // Iterate over the input string by 2 characters at a time
//     let mut chars = hex_string.chars();
//     while let Some(hex_char1) = chars.next() {
//         if let Some(hex_char2) = chars.next() {
//             // Combine two hexadecimal characters into a substring
//             let hex_pair = format!("{}{}", hex_char1, hex_char2);

//             // Parse the hexadecimal substring into a u8
//             match u8::from_str_radix(&hex_pair, 16) {
//                 Ok(byte) => bytes.push(byte),
//                 Err(err) => {
//                     eprintln!("Error parsing hexadecimal string: {:?}", err);
//                     panic!("Failed to parse hexadecimal string");
//                 }
//             }
//         } else {
//             panic!("Unexpected end of input");
//         }
//     }

//     bytes
// }

// fn extract_hashes_from_tree_content(tree_content: &str) -> Vec<String> {
//     let mut hashes = Vec::new();

//     let mut chars = tree_content.chars();
//     while let Some(c) = chars.next() {
//         // Check if the current character is a hexadecimal digit
//         println!("{}", c);
//         if c.is_ascii_hexdigit() {
//             // Collect the hexadecimal string
//             let mut hash = String::new();
//             hash.push(c);

//             // Continue collecting characters until a non-hexadecimal digit is encountered
//             while let Some(next_c) = chars.next() {
//                 if next_c.is_ascii_hexdigit() {
//                     hash.push(next_c);
//                 } else {
//                     break;
//                 }
//             }

//             // Check if the collected string is of expected length
//             if hash.len() == 40 {
//                 hashes.push(hash);
//             }
//         } else {
//             // Skip non-hexadecimal characters
//         }
//     }

//     hashes
// }

// pub fn hex_string_to_bytes(bytes: &[u8]) -> String {
//     let mut hash: String = String::new();
//     for byte in bytes {
//         // println!("{:x}", byte);
//         hash.push_str(&format!("{:x}", byte));
//     }

//     hash
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //env::set_var(RELATIVE_PATH, "src/client/");
     // Set an environment variable
     // Retrieve the value of an environment variable
     /*if let Ok(value) = env::var(commands::commands::RELATIVE_PATH) {
         println!("MY_VARIABLE is set to: {}", value);
         println!("relative path: {}",commands::commands::PathHandler::get_relative_path(commands::commands::R_HEADS));
     } else {
         println!("MY_VARIABLE is not set.");
     }*/
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let command = &args[1];
        match command.as_str() {
            "init" => commands::Init::new().execute(None)?,
            "branch" => commands::Branch::new().execute(parse_arguments(&args[2..]))?,
            "checkout" => commands::Checkout::new().execute(parse_arguments(&args[2..]))?,
            "cat-file" => commands::CatFile::new().execute(parse_arguments(&args[2..]))?,
            "hash-object" => commands::HashObject::new().execute(parse_arguments(&args[2..]))?,
            "add" => commands::Add::new().execute(parse_arguments(&args[2..]))?,
            "rm" => commands::Rm::new().execute(parse_arguments(&args[2..]))?,
            "commit" => commands::Commit::new().execute(parse_arguments(&args[2..]))?,
            "status" => commands::Status::new().execute(None)?,
            "log" => commands::Log::new().execute(parse_arguments(&args[2..]))?,
            "remote" => commands::Remote::new().execute(parse_arguments(&args[2..]))?,
            "pack-objects" => commands::PackObjects::new().execute(None)?,
            "fetch" => commands::Fetch::new().execute(parse_arguments(&args[2..]))?,
            "merge" => commands::Merge::new().execute(parse_arguments(&args[2..]))?,
            "clone" => commands::Clone::new().execute(parse_arguments(&args[2..]))?,
            "pull" => commands::Pull::new().execute(None)?,
            "push" => commands::Push::new().execute(None)?,
            "ls-tree" => commands::LsTree::new().execute(parse_arguments(&args[2..]))?,
            "ls-files" => commands::LsFiles::new().execute(parse_arguments(&args[2..]))?,
            "tag" => commands::Tag::new().execute(parse_arguments(&args[2..]))?,
            "check-ignore" => commands::CheckIgnore::new().execute(parse_arguments(&args[2..]))?,
            "show-ref" => commands::ShowRef::new().execute(parse_arguments(&args[2..]))?,
            "unpack-objects" => commands::UnpackObjects::new().execute(parse_arguments(&args[2..]))?,

            _ => return Err(Box::new(io::Error::new(io::ErrorKind::Other,"Error: Invalid command."))),
        };
    };
    Ok(())
}