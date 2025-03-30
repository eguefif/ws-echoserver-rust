use base64::prelude::*;
use rand::Rng;
use sha1::{Digest, Sha1};
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct WebSocketClient {
    socket: TcpStream,
}

impl WebSocketClient {
    pub fn new(ip: &str, port: i32) -> Self {
        let mut socket = TcpStream::connect(format!("{}:{}", ip, port))
            .expect("Error: impossible to connect to remote");
        let key = generate_key();
        let request = build_request(&key);
        socket.write_all(request.as_bytes()).unwrap();
        check_server_response(&mut socket, key);
        println!("Handshake done");
        Self { socket }
    }

    pub fn send_frame(&mut self, payload: &str) {
        println!("Sending: {payload}");
    }

    pub fn read_frame(&mut self) -> String {
        "Hello World".to_string()
    }
}

fn generate_key() -> String {
    let mut key = [0u8; 16];
    rand::rng().fill(&mut key);
    BASE64_STANDARD.encode(key)
}

fn build_request(key: &str) -> String {
    format!(
        " GET / HTTP/1.1\r\n\
Sec-WebSocket-Version: 13\r\n\
Sec-WebSocket-Key: {key}\r\n\
Connection: Upgrade\r\n\
Upgrade: websocket\r\n\
Host: 127.0.0.1:8000\r\n\r\n
    "
    )
}

fn check_server_response(socket: &mut TcpStream, key: String) {
    let response = read_server_http_response(socket).unwrap();
    let client_key = extract_client_key(&response);
    let control_key = get_control_key(&key);
    if client_key != control_key {
        panic!("Error: wrong server Sec-WebSocket-Accept key")
    }
}

fn read_server_http_response(socket: &mut TcpStream) -> Option<String> {
    let mut buffer = vec![0; 1024];
    let mut response = String::new();
    loop {
        if let Ok(_) = socket.read(&mut buffer) {
            let chunk = String::from_utf8_lossy(&buffer);
            if chunk.contains("\r\n\r\n") {
                let chunk = chunk.split("\r\n\r\n").next().unwrap();
                response.push_str(&chunk);
                return Some(response);
            }
            response.push_str(&chunk);
        } else {
            break;
        }
    }
    None
}

fn extract_client_key(response: &str) -> String {
    for line in response.lines() {
        if line.contains("Sec-WebSocket-Accept") {
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

fn get_control_key(key: &str) -> String {
    let guid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let concatenated_key = format!("{}{}", key, guid);
    let mut hasher = Sha1::new();
    hasher.update(concatenated_key);
    let digest = hasher.finalize();
    BASE64_STANDARD.encode(digest)
}
