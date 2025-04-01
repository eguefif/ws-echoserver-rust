use crate::websocket::WebSocket;
use crate::websocket_error::WsError;
use base64::prelude::*;
use sha1::{Digest, Sha1};
use std::error;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct WebSocketServer {
    websocket: WebSocket,
}

impl WebSocketServer {
    pub fn new(mut socket: TcpStream) -> Result<Self, Box<dyn error::Error>> {
        let client_request = read_http_request(&mut socket);
        let key = extract_client_key(&client_request)?;
        let response = build_response(key);
        socket.write_all(response.as_bytes())?;
        Ok(Self {
            websocket: WebSocket::new(socket),
        })
    }

    pub fn try_read_frame(&mut self) -> Result<String, Box<dyn error::Error>> {
        self.websocket.read_frame()
    }

    pub fn send_frame(&mut self, payload: String) -> Result<(), Box<dyn error::Error>> {
        self.websocket.write_frame(payload)
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

fn extract_client_key(response: &str) -> Result<String, Box<dyn error::Error>> {
    for line in response.lines() {
        if line.contains("Sec-WebSocket-Key") {
            if let Some((_, value)) = line.split_once(":") {
                return Ok(value.trim().to_string());
            } else {
                return Err(Box::new(WsError::Header(line.to_string())));
            }
        }
    }
    Err(Box::new(WsError::MissingSecWebSocketAcceptHeader))
}

fn build_response(key: String) -> String {
    let server_key = process_key(&key);
    format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: {}\r\n\r\n\
            ",
        server_key
    )
}

pub fn process_key(key: &str) -> String {
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
