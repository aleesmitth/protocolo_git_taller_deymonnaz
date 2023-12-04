use std::{
    error::Error, io, io::Read,
};


const LENGTH_BYTES: usize = 4;
pub const REQUEST_DELIMITER_DONE: &str = "done\n";
pub const REQUEST_LENGTH_CERO: &str = "0000";
pub const NAK_RESPONSE: &str = "NAK\n";
pub const WANT_REQUEST: &str = "want";
pub const UNPACK_CONFIRMATION: &str = "unpack ok\n";



pub fn format_line_to_send(line: String) -> String {
    format!("{:04x}{}", line.len() + 4, line)
}

pub fn read_until(
    reader: &mut dyn Read,
    delimiter: &str,
    stop_when_length_cero: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut requests_received: Vec<String> = Vec::new();
    loop {
        println!("waiting for request..");
        let request_length = get_request_length(reader)?;
        println!("request length: {:?}", request_length);
        if request_length == 0 {
            if stop_when_length_cero {
                break;
            } else {
                continue;
            }
        }
        println!("reading request..");
        let request = read_exact_length_to_string(reader, request_length)?;
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

pub fn read_exact_length_to_string(
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

pub fn get_request_length(reader: &mut dyn Read) -> Result<usize, Box<dyn std::error::Error>> {
    let mut message_length: [u8; LENGTH_BYTES] = [0; LENGTH_BYTES];
    if let Err(_e) = reader.read_exact(&mut message_length) {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Invalid length in line",
        )));
    }
    let hex_string = String::from_utf8_lossy(&message_length);
    Ok(u32::from_str_radix(&hex_string, 16)? as usize)
}
