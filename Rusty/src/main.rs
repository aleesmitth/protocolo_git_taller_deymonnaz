mod commands;

use crate::commands::structs::Head;
use crate::commands::Command;
use std::{env, io};

/// This function takes a slice of strings and converts it into a vector of string slices.
/// Returns an `Option` containing a `Vec` of string slices if `args` is not empty. Returns `None` if `args` is empty.
fn parse_arguments(args: &[String]) -> Option<Vec<&str>> {
    if args.is_empty() {
        return None;
    }
    let arg_slices: Vec<&str> = args.iter().map(String::as_str).collect();
    Some(arg_slices)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut head = Head::new();
    // let args: Vec<String> = env::args().collect();
    // if args.len() >= 2 {
    //     let command = &args[1];
    //     match command.as_str() {
    //         "init" => commands::Init::new().execute(&mut head, None)?,
    //         "branch" => commands::Branch::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "checkout" => commands::Checkout::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "cat-file" => commands::CatFile::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "hash-object" => commands::HashObject::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "add" => commands::Add::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "rm" => commands::Rm::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "commit" => commands::Commit::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "status" => commands::Status::new().execute(&mut head, None)?,
    //         "log" => commands::Log::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "remote" => commands::Remote::new().execute(&mut head, parse_arguments(&args[2..]))?,
    //         "pack-objects" => commands::PackObjects::new().execute(&mut head, None)?,
    //         _ => return Err(Box::new(io::Error::new(io::ErrorKind::Other,"Error: Invalid command."))),
    //     };
    // };
    //commands::Init::new().execute(&mut head, None)?;
    commands::Commit::new().execute(&mut head, None)?;
    commands::Push::new().execute(&mut head, None)?;
    Ok(())
}

