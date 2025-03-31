use std::fmt;

#[derive(Debug, Clone)]
pub enum WsError {
    InvalidHandshakeKey,
    MissingSecWebSocketAcceptHeader,
    Header(String),
}

impl std::error::Error for WsError {}

impl fmt::Display for WsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WsError::InvalidHandshakeKey => write!(f, "Invalid key for handhsake"),
            WsError::Header(header) => write!(f, "Error in http header: {}", header),
            WsError::MissingSecWebSocketAcceptHeader => {
                write!(f, "Server respond without a Sec-WebSocketAccept header")
            }
        }
    }
}
