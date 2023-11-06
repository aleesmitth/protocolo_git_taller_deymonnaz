use std::{fs, error::Error, io::Write, fmt, net::TcpStream, io::BufRead};
const OBJECT: &str = ".git/objects";
const INDEX_FILE: &str = ".git/index";
const DEFAULT_REMOTE: &str = "origin";

use crate::commands::helpers;


use super::helpers::get_file_length;

/// Abstract struct for creating new objects in git repository
pub struct HashObjectCreator;

impl HashObjectCreator {
    fn new() -> Self {
        HashObjectCreator {}
    }

    /// Writes an object file to the Git repository.
    ///
    /// This function takes the provided content, object type, and file length, and writes the object
    /// data to a file in the Git repository. The content is first formatted with object type and file
    /// length, hashed, and then compressed before being written to the repository.
    /// Returns a Result that may contain a string of the hash of the written object.
    pub fn write_object_file(content: String, obj_type: ObjectType, file_len: u64) -> Result<String, Box<dyn Error>> {
        let data = format!("{} {}\0{}", obj_type, file_len, content);
        let hashed_data = Self::generate_object_hash(obj_type, file_len, &content);
        let compressed_content = helpers::compress_content(data.as_str())?;
        let obj_directory_path = format!("{}/{}", OBJECT, &hashed_data[0..2]);
        let _ = fs::create_dir(&obj_directory_path);
    
        let object_file_path = format!("{}/{}", obj_directory_path, &hashed_data[2..]);
        if fs::metadata(object_file_path.clone()).is_ok() {
            return Ok(hashed_data)
        }
        
        let mut object_file = fs::File::create(object_file_path.clone())?;
        object_file.write_all(&compressed_content)?;

        Ok(hashed_data)
    }

    pub fn generate_object_hash(obj_type: ObjectType, file_len: u64, content: &str) -> String {
        let data = format!("{} {}\0{}", obj_type, file_len, content);
        helpers::generate_sha1_string(data.as_str())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum FileStatus {
    Untracked,
    Modified,
    Staged,
}

/// Represents the staging area for Git. Where files can be added and removed. They can have 3 possible states,
/// stage, modified and untracked. 
#[derive(Debug)]
pub struct StagingArea;

impl StagingArea {
    pub fn new() -> Self {
        StagingArea {}
    }

    /// Adds a file to the staging area. Creating a git object and saving the object's path, hash and state in the
    /// index file, following the format: file_path;hash;state.
    pub fn add_file(&self, _head: &mut Head, path: &str) -> Result<(), Box<dyn Error>> {
        // let hash_object = HashObject::new();
        // let object_hash = hash_object.execute(head, Some(&["-w", path]))?;
        let file_content = helpers::read_file_content(path)?;
        let object_hash = HashObjectCreator::write_object_file(file_content, ObjectType::Blob, get_file_length(path)?)?;
        //no se si aca esta bien escribir directamente el objeto o seria mejor usar hash-object

        helpers::update_file_with_hash(&object_hash.as_str(), "2", path)?;

        Ok(())
    }

    /// Removes a file from the staging area.
    pub fn remove_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        helpers::remove_object_from_file(path)?;
        Ok(())
    }

    pub fn unstage_index_file(&self) -> Result<(), Box<dyn Error>> {
        let index_file_content = helpers::read_file_content(INDEX_FILE)?;
        let mut lines: Vec<String> = index_file_content.lines().map(|s| s.to_string()).collect();
        let mut new_index_file_content = String::new();

        for line in lines.iter_mut() {
            line.pop();
            line.push_str("0");
            new_index_file_content.push_str(line);
            new_index_file_content.push('\n'); // Add a newline between lines
        } 
        
        let mut index_file = fs::File::create(INDEX_FILE)?;
        index_file.write_all(new_index_file_content.as_bytes())?;
        Ok(())
    }

    // fn unstage_file(&mut self, path: &str) {
    //     if let Some(status) = self.files.get_mut(path) {
    //         *status = FileStatus::Modified;
    //     }
    // }

    // fn list_stagedfiles(&self) -> Vec<&str> {
    //     self.files
    //         .iter()
    //         .filter(|&(, status)|status == FileStatus::Staged)
    //         .map(|(path, _)| path.as_str())
    //         .collect()
    // }
}

pub struct Head {
	branches: Vec<String>
}

impl Head {	
	pub fn new() -> Self {
		Head { branches: Vec::new() }
	}

	pub fn add_branch(&mut self, name: &str) {
		// Check if the branch name is not already in the vector
	    if !self.branches.iter().any(| branch | branch == name) {
	        self.branches.push(name.to_string());
	    }
	}

	pub fn delete_branch(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
	    // Use the retain method to remove branches with the specified name
	    self.branches.retain( | branch | branch != name);

	    Ok(())
	}

	pub fn rename_branch(&mut self, old_name: &str, new_name: &str) -> Result<(), Box<dyn Error>> {
	    // Find the branch with the old name and rename it to the new name
	    if let Some(branch) = self.branches.iter_mut().find( | branch | *branch == old_name) {
	        *branch = new_name.to_string();
	    }

	    Ok(())
	}

	pub fn print_all(&self) {
		for s in self.branches.iter() {
			println!("branch:{}", s);
		}
	}
}

/// Represents the type of a Git object.
pub enum ObjectType {
    Blob,
    Commit,
    Tree,
    Tag,
}

impl ObjectType {
    pub fn new(obj_type: &str) -> Option<Self> {
        match obj_type {
            "blob" => Some(ObjectType::Blob),
            "commit" => Some(ObjectType::Commit),
            "tree" => Some(ObjectType::Tree),
            "tag" => Some(ObjectType::Tag),
            _ => None,
        }
    }
    pub fn get_object_for_pack_file(&self) -> u8 {
        let object_type = match self {
            ObjectType::Commit => 1,
            ObjectType::Tree => 2,
            ObjectType::Blob => 3,
            ObjectType::Tag => 4,
        };
        object_type
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            ObjectType::Blob => "blob",
            ObjectType::Commit => "commit",
            ObjectType::Tree => "tree",
            ObjectType::Tag => "tag",
        };
        write!(f, "{}", string)
    }
}

pub struct ServerConnection;

impl ServerConnection {
    pub fn new() -> Self {
        ServerConnection {}
    }

    pub fn receive_pack(&mut self) -> Result<(), Box<dyn Error>> {
        println!("1");
        //let remote_server_address = helpers::get_remote_url(DEFAULT_REMOTE)?;
        //let mut stream = TcpStream::connect(remote_server_address)?;
        let mut stream = TcpStream::connect("127.0.0.1:9418")?;

        let service = "git-receive-pack /.git\0host=127.0.0.1\0";
        let request = format!("{:04x}{}", service.len() + 4, service);
        // Send the Git service request
        stream.write_all(request.as_bytes())?;

        // Read the response from the server
        let mut response = String::new();
        {
        let mut reader = std::io::BufReader::new(&stream);
        for line in reader.lines() {
            let line = line?;
            println!("line: {}", line);
            break;
        }
        }
        println!("{:?}", response);

        let branch_path = helpers::get_current_branch_path()?;
        let last_commit_hash: String = helpers::read_file_content(&branch_path)?;
        println!("last_commit: {}", last_commit_hash);
        let line = format!("0000000000000000000000000000000000000000 {} refs/heads/new", last_commit_hash);
        let actual_line = format!("{:04x}{}\n", line.len() + 5, line);
        println!("line: {}", actual_line);
        stream.write_all(actual_line.as_bytes())?;
        stream.write_all(b"0000")?;
        
        let mut pack_file = fs::File::open(".git/pack/pack_file.pack")?;
        std::io::copy(&mut pack_file, &mut stream)?;
        //stream.flush()?;

        response.clear();
        let mut reader = std::io::BufReader::new(&stream);
        for line in reader.lines() {
            let line = line?;
            println!("line: {}", line);
            break;
        }

        Ok(())
    }

    pub fn clone_from_remote(&self) -> Result<(), Box<dyn Error>> {
        let mut stream = TcpStream::connect("127.0.0.1:9418")?;

        let request = format!("{:04x}git-upload-pack /.git\0host=127.0.0.1\0", "git-upload-pack /.git\0host=127.0.0.1\0".len() + 4);
        stream.write_all(request.as_bytes())?;
        stream.flush()?;

        let reader = std::io::BufReader::new(&stream);
        for line in reader.lines() {
            let line = line?;
            println!("{}", line);
            break;
        }

        let branch_path = helpers::get_current_branch_path()?;
        let last_commit_hash: String = helpers::read_file_content(&branch_path)?;

        let line = format!("want {} multi_ack side-band-64k ofs-delta", last_commit_hash);
        let actual_line = format!("{:04x}{}\n", line.len() + 5, line);
        println!("{}", actual_line);
        stream.write_all(actual_line.as_bytes())?;
        stream.write_all("0000".as_bytes())?;
        let done = format!("{:04x}done\n", "done\n".len()+4);
        println!("{}", done);
        stream.write_all(done.as_bytes())?;
        stream.flush()?;

        Ok(())
        }
}




