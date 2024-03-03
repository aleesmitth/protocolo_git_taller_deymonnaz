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
        let path_handler = PathHandler::new(String::new());
        match command.as_str() {
            "init" => Init::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "branch" => Branch::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "checkout" => Checkout::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "cat-file" => CatFile::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "hash-object" => {
                HashObject::new().execute(parse_arguments(&args[2..]), &path_handler)?
            }
            "add" => Add::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "rm" => Rm::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "commit" => Commit::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "status" => Status::new().execute(None, &path_handler)?,
            "log" => Log::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "remote" => Remote::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "pack-objects" => {
                PackObjects::new().execute(parse_arguments(&args[2..]), &path_handler)?
            }
            "fetch" => Fetch::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "merge" => Merge::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "clone" => Clone::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "pull" => Pull::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "push" => Push::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "ls-tree" => LsTree::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "ls-files" => LsFiles::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "tag" => Tag::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "check-ignore" => {
                CheckIgnore::new().execute(parse_arguments(&args[2..]), &path_handler)?
            }
            "show-ref" => ShowRef::new().execute(parse_arguments(&args[2..]), &path_handler)?,
            "unpack-objects" => {
                UnpackObjects::new().execute(parse_arguments(&args[2..]), &path_handler)?
            }
            "rebase" => Rebase::new().execute(parse_arguments(&args[2..]), &path_handler)?,

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
