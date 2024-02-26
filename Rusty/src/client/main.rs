use rusty::commands::git_commands;
use rusty::commands::git_commands::Command;
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
            "init" => git_commands::Init::new().execute(None)?,
            "branch" => git_commands::Branch::new().execute(parse_arguments(&args[2..]))?,
            "checkout" => git_commands::Checkout::new().execute(parse_arguments(&args[2..]))?,
            "cat-file" => git_commands::CatFile::new().execute(parse_arguments(&args[2..]))?,
            "hash-object" => {
                git_commands::HashObject::new().execute(parse_arguments(&args[2..]))?
            }
            "add" => git_commands::Add::new().execute(parse_arguments(&args[2..]))?,
            "rm" => git_commands::Rm::new().execute(parse_arguments(&args[2..]))?,
            "commit" => git_commands::Commit::new().execute(parse_arguments(&args[2..]))?,
            "status" => git_commands::Status::new().execute(None)?,
            "log" => git_commands::Log::new().execute(parse_arguments(&args[2..]))?,
            "remote" => git_commands::Remote::new().execute(parse_arguments(&args[2..]))?,
            "pack-objects" => {
                git_commands::PackObjects::new().execute(parse_arguments(&args[2..]))?
            }
            "fetch" => git_commands::Fetch::new().execute(parse_arguments(&args[2..]))?,
            "merge" => git_commands::Merge::new().execute(parse_arguments(&args[2..]))?,
            "clone" => git_commands::Clone::new().execute(parse_arguments(&args[2..]))?,
            "pull" => git_commands::Pull::new().execute(parse_arguments(&args[2..]))?,
            "push" => git_commands::Push::new().execute(parse_arguments(&args[2..]))?,
            "ls-tree" => git_commands::LsTree::new().execute(parse_arguments(&args[2..]))?,
            "ls-files" => git_commands::LsFiles::new().execute(parse_arguments(&args[2..]))?,
            "tag" => git_commands::Tag::new().execute(parse_arguments(&args[2..]))?,
            "check-ignore" => {
                git_commands::CheckIgnore::new().execute(parse_arguments(&args[2..]))?
            }
            "show-ref" => git_commands::ShowRef::new().execute(parse_arguments(&args[2..]))?,
            "unpack-objects" => {
                git_commands::UnpackObjects::new().execute(parse_arguments(&args[2..]))?
            }
            "rebase" => git_commands::Rebase::new().execute(parse_arguments(&args[2..]))?,

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
