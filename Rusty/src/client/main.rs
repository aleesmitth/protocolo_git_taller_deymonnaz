use rusty::commands::commands::Command;
use rusty::commands::commands;
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
    // fs::File::create("file.txt");
    // fs::create_dir("dir");
    // fs::File::create("dir/1.txt");
    // fs::remove_dir_all(".git/");
    // commands::Init::new().execute(None)?;
    // commands::Add::new().execute(Some(vec!["file.txt"]))?;
    // commands::Commit::new().execute(None)?;
    // commands::Branch::new().execute(Some(vec!["new"]))?;
    // commands::Checkout::new().execute(Some(vec!["new"]))?;
    // commands::Add::new().execute(Some(vec!["dir/1.txt"]))?;
    // commands::Commit::new().execute(None)?;
    // commands::Checkout::new().execute(Some(vec!["main"]))?;
    // commands::Checkout::new().execute(Some(vec!["new"]))?;
    // commands::PackObjects::new().execute(Some(vec!["466fdf0f96d9264eddcc17a9b9a3d688ab45e73f"]))?;
    // commands::UnpackObjects::new().execute(Some(vec![".git/pack/pack_file.pack"]))?;
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