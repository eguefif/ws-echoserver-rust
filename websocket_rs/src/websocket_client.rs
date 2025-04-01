use crate::websocket::WebSocket;
use crate::websocket_error::WsError;
use crate::websocket_server::process_key;
use base64::prelude::*;
use rand::Rng;
use std::error;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct WebSocketClient {
    websocket: WebSocket,
}

impl WebSocketClient {
    pub fn new(ip: &str, port: i32) -> Result<Self, Box<dyn error::Error>> {
        let mut socket = TcpStream::connect(format!("{}:{}", ip, port))?;
        let key = generate_key();
        let request = build_request(&key);
        socket.write_all(request.as_bytes())?;
        check_server_response(&mut socket, key)?;
        Ok(Self {
            websocket: WebSocket::new(socket),
        })
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

fn check_server_response(socket: &mut TcpStream, key: String) -> Result<(), Box<dyn error::Error>> {
    let response = read_server_http_response(socket)?;
    let client_key = extract_server_key(&response)?;
    let control_key = get_control_key(&key);
    if client_key == control_key {
        Ok(())
    } else {
        Err(Box::new(WsError::InvalidHandshakeKey))
    }
}

fn read_server_http_response(socket: &mut TcpStream) -> std::io::Result<String> {
    let mut buffer = vec![0; 1024];
    let mut response = String::new();
    loop {
        socket.read(&mut buffer)?;
        let chunk = String::from_utf8_lossy(&buffer);
        if chunk.contains("\r\n\r\n") {
            let chunk = chunk.split("\r\n\r\n").next().unwrap();
            response.push_str(&chunk);
            return Ok(response);
        }
        response.push_str(&chunk);
    }
}

fn extract_server_key(response: &str) -> Result<String, Box<dyn error::Error>> {
    for line in response.lines() {
        if line.contains("Sec-WebSocket-Accept") {
            if let Some((_, value)) = line.split_once(":") {
                return Ok(value.trim().to_string());
            } else {
                return Err(Box::new(WsError::Header(line.to_string())));
            }
        }
    }
    Err(Box::new(WsError::MissingSecWebSocketAcceptHeader))
}

fn get_control_key(key: &str) -> String {
    process_key(key)
}
