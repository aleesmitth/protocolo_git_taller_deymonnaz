use crate::commands::helpers;
use std::{
    env, error::Error, fs, io, io::BufRead, io::Read, io::Write, net::TcpListener, net::TcpStream,
    str,
};
const RECEIVE_PACK: &str = "git-receive-pack";
const UPLOAD_PACK: &str = "git-upload-pack";
pub struct ServerProtocol;
const LENGTH_BYTES: usize = 4;
const WANT_REQUEST: &str = "want";
const REQUEST_LENGTH_CERO: &str = "0000";
const REQUEST_DELIMITER_DONE: &str = "done\n";

impl ServerProtocol {
    pub fn new() -> Self {
        ServerProtocol {}
    }

    pub fn bind(address: &str) -> Result<TcpListener, Box<dyn std::error::Error>> {
        println!("binding to client...");
        Ok(TcpListener::bind(address)?)
    }

    fn get_request_length(reader: &mut dyn Read) -> Result<usize, Box<dyn std::error::Error>> {
        let mut message_length: [u8; LENGTH_BYTES] = [0; LENGTH_BYTES];
        if let Err(e) = reader.read_exact(&mut message_length) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Invalid length in line",
            )));
        }
        let hex_string = String::from_utf8_lossy(&message_length);
        Ok(u32::from_str_radix(&hex_string, 16)? as usize)
    }

    // fn read_message_to_buffer(reader: &mut dyn Read, message_length: u32) -> Result<[u8; message_length], Box<dyn std::error::Error>> {
    //     let mut buffer: [u8; message_length] = [0; message_length];
    //     if let Err(e) = reader.read_exact(&mut buffer) {
    //         return Err(Box::new(io::Error::new(
    //             io::ErrorKind::Other,
    //             "Error reading request",
    //         )));
    //     }
    //     Ok(buffer)
    // }

    fn read_exact_length_to_string(
        reader: &mut dyn Read,
        message_length: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = vec![0; message_length - 4];
        if let Err(e) = reader.read_exact(&mut buffer) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Error reading request: {}", e),
            )));
        }
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    pub fn handle_client_conection(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        // In the Git Dumb Protocol, Git commands are sent as text lines.
        // You would parse the incoming lines and respond accordingly.
        let stream_clone = stream.try_clone()?;
        let mut reader = std::io::BufReader::new(stream_clone);
        println!("waiting for request...");
        let request_length = ServerProtocol::get_request_length(&mut reader)?;
        println!("request_length: {:?}", request_length);
        if request_length == 0 {
            // TODO gracefully end connection, message length is 0
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Error: Request length 0",
            )));
        }
        //println!("{:x}", ServerProtocol::read_message_length(&mut reader).unwrap());
        // let mut writer = std::io::BufWriter::new(stream);
        println!("reading request_...");
        // let mut buffer = [0; 1024];
        // let _ = stream.read(&mut buffer);
        // println!("{:?}", buffer);
        let request = ServerProtocol::read_exact_length_to_string(&mut reader, request_length)?;
        println!("request: {:?}", request);
        let request_array: Vec<&str> = request.split_whitespace().collect();
        println!("request in array: {:?}", request_array);
        match request_array[0] {
            UPLOAD_PACK => ServerProtocol::upload_pack(stream)?,
            RECEIVE_PACK => println!("push: {:?}", request_array),
            _ => {}
        }

        // for byte in reader.bytes() {
        //     match byte {
        //         Ok(b'\0') => {
        //             // '\0' encountered, end of data
        //             break;
        //         }
        //         Ok(byte) => {
        //             // Handle the byte (e.g., print it)
        //             println!("Received byte: {}", byte as char);
        //         }
        //         Err(e) => {
        //             eprintln!("Error reading byte: {}", e);
        //             break;
        //         }
        //     }
        // }
        /*for line in reader.bytes() {
            if let Ok(line) = line {
                println!("incoming line: {:?}", line);
                // writer.write_all(b"# service=git-upload-pack\n").unwrap();
                // writer.flush().unwrap();
                /*if line.starts_with("git-upload-pack") {
                    println!("upload pack received");
                    writer.write_all(b"# service=git-upload-pack\n").unwrap();
                    writer.flush().unwrap();
                } else if line.starts_with("git-receive-pack") {
                    println!("git-receive-pack");
                    writer.write_all(b"# service=git-receive-pack\n").unwrap();
                    writer.flush().unwrap();
                } else if line == "capabilities" {
                    writer.write_all(b"delete-refs side-band-64k\n").unwrap();
                    writer.write_all(b"multi_ack\n").unwrap();
                    writer.write_all(b"side-band\n").unwrap();
                    writer.write_all(b"ofs-delta\n").unwrap();
                    writer.write_all(b"thin-pack\n").unwrap();
                    writer.write_all(b"shallow\n").unwrap();
                    writer.write_all(b"no-progress\n").unwrap();
                    writer.write_all(b"include-tag\n").unwrap();
                    writer.write_all(b"multi_ack_detailed\n").unwrap();
                    writer.flush().unwrap();
                }*/
            } else {
                break;
            }
        }*/

        println!("end handling connection");
        Ok(())
    }

    pub fn upload_pack(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        println!("git-upload-pack");
        let branches: Vec<String> = helpers::get_all_branches()?;
        for branch in &branches {
            let line_to_send = ServerProtocol::format_line_to_send(branch.clone());
            println!("sending line: {:?}", line_to_send);
            stream.write_all(line_to_send.as_bytes())?;
        }

        stream.write_all(REQUEST_LENGTH_CERO.as_bytes());
        println!("-sent end of message delimiter-");

        let mut reader = std::io::BufReader::new(stream);
        let requests_received: Vec<String> =
            ServerProtocol::read_until(&mut reader, REQUEST_DELIMITER_DONE, false)?;
        for request_received in requests_received {
            let request_array: Vec<&str> = request_received.split_whitespace().collect();
            println!("request in array: {:?}", request_array);
            if request_array[0] != WANT_REQUEST {
                //TODO not want request, handle error gracefully
                println!(
                    "Error: expected want request but got: {:?}",
                    request_array[0]
                );
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Expecting want request but got something else",
                )));
            }
            // TODO should probably go read the branches again in case some other client updated the latest commit
            let is_valid_commit =
                ServerProtocol::validate_is_latest_commit_any_branch(request_array[1], &branches);
            if !is_valid_commit {
                println!("invalid commit: {:?}", request_array);
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Error: Received invalid commit hash for want request",
                )));
            }

            println!("valid want request.");
        }

        Ok(())
    }

    pub fn read_until(
        reader: &mut dyn Read,
        delimiter: &str,
        stop_when_length_cero: bool,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut requests_received: Vec<String> = Vec::new();
        while true {
            println!("waiting for request..");
            let request_length = ServerProtocol::get_request_length(reader)?;
            println!("request length: {:?}", request_length);
            if request_length == 0 {
                if stop_when_length_cero {
                    break;
                } else {
                    continue;
                }
            }
            println!("reading request..");
            let request = ServerProtocol::read_exact_length_to_string(reader, request_length)?;
            println!("request: {:?}", request);

            // received a message delimiter
            if &request == delimiter {
                println!("found delimiter {:?}", delimiter);
                break;
            }
            requests_received.push(request);
        }
        Ok(requests_received)
    }

    pub fn validate_is_latest_commit_any_branch(commit: &str, branches: &Vec<String>) -> bool {
        for branch in branches {
            // Split the string into words
            let branch_commit_and_name: Vec<&str> = branch.split_whitespace().collect();
            if let Some(first_word) = branch_commit_and_name.first() {
                if first_word == &commit {
                    return true;
                }
            }
        }
        false
    }

    pub fn format_line_to_send(line: String) -> String {
        format!("{:04x}{}", line.len() + 4, line)
    }

    pub fn receive_pack(&mut self) -> Result<(), Box<dyn Error>> {
        /*println!("1");
        //let remote_server_address = helpers::get_remote_url(DEFAULT_REMOTE)?;
        let mut stream = TcpStream::connect("127.0.0.1:9418")?;

        let service = "git-receive-pack /.git\0host=127.0.0.1\0";
        let request = format!("{:04x}{}", service.len() + 4, service);
        // Send the Git service request
        stream.write_all(request.as_bytes())?;

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
        //stream.flush()?;


        let reader = std::io::BufReader::new(&stream);
        for line in reader.lines() {
            let line = line?;
            println!("response line: {}", line);
            break;
        }

        Ok(())*/
        Ok(())
    }

    pub fn clone_from_remote(&self) -> Result<(), Box<dyn Error>> {
        /*let mut stream = TcpStream::connect("127.0.0.1:9418")?;

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

        Ok(())*/
        Ok(())
    }
}
