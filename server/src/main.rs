use std::net::{TcpListener, TcpStream};
use std::thread;
use websocket_rs::websocket_server::WebSocketServer;

fn main() -> std::io::Result<()> {
    run_server("127.0.0.1", 9000)?;
    Ok(())
}

fn run_server(ip: &str, port: u32) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", ip, port))?;
    for stream in listener.incoming() {
        match stream {
            Ok(socket) => {
                thread::spawn(move || handle_client(socket));
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    Ok(())
}

fn handle_client(socket: TcpStream) {
    let mut ws_server = WebSocketServer::new(socket).unwrap();
    loop {
        let payload = ws_server.try_read_frame().unwrap();
        if payload.len() > 0 {
            ws_server.send_frame(payload).unwrap();
        }
    }
}
