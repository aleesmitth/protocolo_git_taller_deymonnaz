use crate::commands::helpers;
use std::{fs, error::Error, io, io::Write, io::Read, str, env, io::BufRead, net::TcpStream, net::Shutdown};

pub struct ClientProtocol;

impl ClientProtocol {
    pub fn new() -> Self {
        ClientProtocol {}
    }

    pub fn connect(&mut self, address: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
        println!("connecting to server...");
        //let remote_server_address = helpers::get_remote_url(DEFAULT_REMOTE)?;
        Ok(TcpStream::connect(address)?)
    }

    pub fn receive_pack(&mut self) -> Result<(), Box<dyn Error>> {
        let mut stream = self.connect("127.0.0.1:9418")?;
        println!("connect complete");

        let service = "git-receive-pack /23C2-Rusty/Rusty/.git\0host=127.0.0.1\0";
        let request = format!("{:04x}{}", service.len() + 4, service);
        println!("request {}", request);
        // Send the Git service request
        stream.write_all(request.as_bytes())?;
        stream.flush();
        println!("request sent");
        // Read the response from the server
        let mut response = String::new();
    
        let reader = std::io::BufReader::new(&stream);
        let mut remote_hash  = String::new();
        for line in reader.lines() {
            if let Ok(value) = line {
                let split_value: Vec<&str> = value.split_whitespace().collect();
                remote_hash = split_value[0].to_string()[4..].to_string();
                println!("response line: {:?}", value);
                println!("remote hash: {}", remote_hash);
            }
            break;
        }

        response.clear();
        let branch_path = helpers::get_current_branch_path()?;
        let last_commit_hash: String = helpers::read_file_content(&branch_path)?;
        println!("last_commit: {}", last_commit_hash);
        let line = format!("{} {} refs/heads/main", remote_hash, last_commit_hash);
        let actual_line = format!("{:04x}{}\n", line.len() + 5, line);
        println!("push line: {}", actual_line);
        stream.write_all(actual_line.as_bytes())?;
        stream.write_all(b"0000")?;
        stream.flush()?;
        
        let mut pack_file = fs::File::open(".git/pack/pack_file.pack")?;
        std::io::copy(&mut pack_file, &mut stream)?;
        // stream.flush()?;

        
        let reader = std::io::BufReader::new(&stream);
        for line in reader.lines() {
            let line = line?;
            println!("response line: {}", line);
            break;
        }

        Ok(())
    }

    pub fn clone_from_remote(&self) -> Result<(), Box<dyn Error>> {
        let mut stream = TcpStream::connect("127.0.0.1:9418")?;

        let request = format!("{:04x}git-upload-pack /23C2-Rusty/Rusty/.git\0host=127.0.0.1\0", "git-upload-pack /23C2-Rusty/Rusty/.git\0host=127.0.0.1\0".len() + 4);
        stream.write_all(request.as_bytes())?;
        stream.flush()?;

        let mut objects_in_remote = Vec::new();
        let reader = std::io::BufReader::new(&stream);
        for line in reader.lines() {
            let line = line?;
            println!("{}", line);
            let split_line: Vec<&str> = line.split_whitespace().collect();
            let object_id = split_line[0].to_string()[4..].to_string();
            println!("{}", object_id);
            objects_in_remote.push(object_id);
            break;
        }

        for object_id in objects_in_remote {
            let line = format!("want {} multi_ack side-band-64k ofs-delta", object_id);
            let actual_line = format!("{:04x}{}\n", line.len() + 5, line);
            println!("{}", actual_line);
            stream.write_all(actual_line.as_bytes())?;
        }
        stream.write_all("0000".as_bytes())?;
        let done = format!("{:04x}done\n", "done\n".len()+4);
        println!("{}", done);
        stream.write_all(done.as_bytes())?;
        stream.flush()?;

        // Read data from the stream
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;

        // Writing the received data into a file
        std::fs::write(".git/pack/received_pack_file.pack", &buffer)?;

        //Closing the connection
        stream.shutdown(Shutdown::Both)?;

        Ok(())
    }
}