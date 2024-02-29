use rusty::commands::git_commands::*;
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
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let command = &args[1];
        match command.as_str() {
            "init" => Init::new().execute(parse_arguments(&args[2..]))?,
            "branch" => Branch::new().execute(parse_arguments(&args[2..]))?,
            "checkout" => Checkout::new().execute(parse_arguments(&args[2..]))?,
            "cat-file" => CatFile::new().execute(parse_arguments(&args[2..]))?,
            "hash-object" => {
                HashObject::new().execute(parse_arguments(&args[2..]))?
            }
            "add" => Add::new().execute(parse_arguments(&args[2..]))?,
            "rm" => Rm::new().execute(parse_arguments(&args[2..]))?,
            "commit" => Commit::new().execute(parse_arguments(&args[2..]))?,
            "status" => Status::new().execute(None)?,
            "log" => Log::new().execute(parse_arguments(&args[2..]))?,
            "remote" => Remote::new().execute(parse_arguments(&args[2..]))?,
            "pack-objects" => {
                PackObjects::new().execute(parse_arguments(&args[2..]))?
            }
            "fetch" => Fetch::new().execute(parse_arguments(&args[2..]))?,
            "merge" => Merge::new().execute(parse_arguments(&args[2..]))?,
            "clone" => Clone::new().execute(parse_arguments(&args[2..]))?,
            "pull" => Pull::new().execute(parse_arguments(&args[2..]))?,
            "push" => Push::new().execute(parse_arguments(&args[2..]))?,
            "ls-tree" => LsTree::new().execute(parse_arguments(&args[2..]))?,
            "ls-files" => LsFiles::new().execute(parse_arguments(&args[2..]))?,
            "tag" => Tag::new().execute(parse_arguments(&args[2..]))?,
            "check-ignore" => {
                CheckIgnore::new().execute(parse_arguments(&args[2..]))?
            }
            "show-ref" => ShowRef::new().execute(parse_arguments(&args[2..]))?,
            "unpack-objects" => {
                UnpackObjects::new().execute(parse_arguments(&args[2..]))?
            }
            "rebase" => Rebase::new().execute(parse_arguments(&args[2..]))?,

            _ => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Invalid command.",
                )))
            }
        };
    };
    Ok(())
}
