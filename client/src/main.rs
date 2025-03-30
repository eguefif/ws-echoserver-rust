use crate::websocketclient::WebSocketClient;
mod websocketclient;

fn main() {
    let mut websocket = WebSocketClient::new("127.0.0.1", 8000);
    println!("Handshake done");
    websocket.send_frame("Hello, World");
    let response = websocket.read_frame();
    println!("Response");
}
