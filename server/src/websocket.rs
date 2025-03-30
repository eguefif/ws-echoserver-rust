use base64::prelude::*;
use sha1::{Digest, Sha1};
use std::io::{Read, Write};
use std::net::TcpStream;

#[allow(dead_code)]
pub struct WebSocket {
    socket: TcpStream,
}

impl WebSocket {
    pub fn new(mut socket: TcpStream) -> Self {
        let client_request = read_http_request(&mut socket);
        let key = extract_client_key(client_request);
        let response = build_response(key);
        socket.write_all(response.as_bytes()).unwrap();
        Self { socket }
    }

    pub fn try_read_frame(&mut self) -> Option<String> {
        Some("Hey".to_string())
    }

    pub fn send_frame(&mut self, payload: String) {
        println!("Sending: {payload}");
    }
}

fn read_http_request(socket: &mut TcpStream) -> String {
    let mut retval = String::new();
    let mut buffer = vec![0; 1024];
    loop {
        if let Ok(_) = socket.read(&mut buffer) {
            let chunk = String::from_utf8_lossy(&buffer);
            if chunk.contains("\r\n\r\n") {
                let mut splits = chunk.split("\r\n\r\n");
                let last_chunk = splits.next().unwrap();
                retval.push_str(last_chunk);
                break;
            } else {
                retval.push_str(&chunk);
            }
        }
    }
    retval
}

fn extract_client_key(client_request: String) -> String {
    for line in client_request.lines() {
        if line.contains("Sec-WebSocket-Key") {
            let mut splits = line.split(":");
            splits.next().expect("Error: header wrong format");
            return splits
                .next()
                .expect("Error: no value for websocket key")
                .trim()
                .to_string();
        }
    }
    panic!("Error: not a valid websocket upgrade request")
}

fn build_response(key: String) -> String {
    let server_key = process_key(&key);
    format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: {}\r\n\r\n
",
        server_key
    )
}

fn process_key(key: &str) -> String {
    let guid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let concatenated_key = format!("{}{}", key, guid);
    let mut hasher = Sha1::new();
    hasher.update(concatenated_key);
    let digest = hasher.finalize();
    BASE64_STANDARD.encode(digest)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_processed_key() {
        let client_key = "dGhlIHNhbXBsZSBub25jZQ==";
        let expected = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";

        let result = process_key(client_key);
        assert_eq!(result, expected);
    }
}
