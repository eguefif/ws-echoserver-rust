use crate::websocketclient::WebSocketClient;
mod websocketclient;

fn main() {
    let mut websocket = WebSocketClient::new("127.0.0.1", 9000);
    println!("Handshake done");
    websocket.send_frame("Hello, World");
    let _ = websocket.read_frame();
}
