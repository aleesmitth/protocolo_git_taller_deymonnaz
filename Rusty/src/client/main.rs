use rusty::commands::commands::CheckIgnore;
use rusty::commands::commands::RELATIVE_PATH;
use rusty::commands::commands::Init;
use rusty::commands::commands::Add;
use rusty::commands::commands::Commit;
use rusty::commands::commands::Push;
use rusty::commands::commands::Clone;
use rusty::commands::commands::ShowRef;
use rusty::commands::structs::Head;
use rusty::commands::commands::Checkout;
use rusty::commands::commands::Branch;
use rusty::commands::commands::Tag;
use rusty::commands::commands::Command;
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
    //env::set_var(RELATIVE_PATH, "src/client/");
     // Set an environment variable
     // Retrieve the value of an environment variable
     /*if let Ok(value) = env::var(commands::commands::RELATIVE_PATH) {
         println!("MY_VARIABLE is set to: {}", value);
         println!("relative path: {}",commands::commands::PathHandler::get_relative_path(commands::commands::R_HEADS));
     } else {
         println!("MY_VARIABLE is not set.");
     }*/
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
    // Init::new().execute(&mut head, None)?;
    // Branch::new().execute(&mut head, Some(vec!["new"]))?;
    // Checkout::new().execute(&mut head, Some(vec!["new"]))?;
    // Add::new().execute(&mut head, Some(vec!["ejemplo.txt"]))?;
    // Commit::new().execute(&mut head, None)?;
    // if let Err(error) = Clone::new().execute(&mut head, None) {
    //     println!("{}", error);
    //     return Ok(())
    // }
    if let Err(error) = Clone::new().execute(&mut head, None) {
        println!("{}", error);
        return Ok(())
    }
    Ok(())
}