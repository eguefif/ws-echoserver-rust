use std::error;
use std::io::Read;
use std::net::TcpStream;

enum Opcode {
    Text,
    None,
}

#[derive(PartialEq)]
enum LenType {
    Single,
    Double,
    Big,
    None,
}
pub struct FrameHandler {
    socket: TcpStream,
    remaining: Vec<u8>,
    len_type: LenType,
    len: usize,
    opcode: Opcode,
    payload: Vec<u8>,
    error: Option<Box<dyn error::Error>>,
}

impl FrameHandler {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            len: 1024,
            len_type: LenType::None,
            opcode: Opcode::None,
            payload: vec![0u8; 1024],
            remaining: vec![0u8; 1024],
            error: None,
        }
    }

    fn get_opcode(&mut self, byte: u8) {
        self.opcode = match byte & 0b00001111 {
            0b1 => Opcode::Text,
            _ => Opcode::None,
        };
    }
    fn get_len_type(&mut self, buffer: &Vec<u8>) {
        let len_byte = buffer[1] & 0b01111111; // first byte is for mask bit
        if buffer.len() > 2 && self.len_type != LenType::None {
            self.len_type = match len_byte {
                126 => LenType::Double,
                127 => LenType::Big,
                _ => LenType::Single,
            }
        }
        match self.len_type {
            LenType::None => {}
            LenType::Single => self.len = buffer[1] as usize,
            LenType::Double => {
                if self.len >= 4 {
                    self.len = ((buffer[3] as usize) << 8 + buffer[4]) as usize;
                }
            }
            LenType::Big => {
                if self.len >= 10 {
                    self.len = ((buffer[5] as usize) << (8 * 5))
                        + ((buffer[6] as usize) << (8 * 4))
                        + ((buffer[7] as usize) << (8 * 3))
                        + ((buffer[8] as usize) << (8 * 2))
                        + (buffer[9] as usize);
                }
            }
        }
    }

    fn header_len(&self) -> usize {
        let len_size = match self.len_type {
            LenType::None => 0,
            LenType::Single => 1,
            LenType::Double => 2,
            LenType::Big => 6,
        };
        len_size + 2
    }
}

impl Iterator for FrameHandler {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        // TODO: see if we need to handle a potential reamining from the last next
        let mut buffer: Vec<u8> = Vec::new();
        while buffer.len() <= (self.len + self.header_len()) && self.len_type == LenType::None {
            let mut tmp = vec![0u8; 1024];
            match self.socket.read(&mut buffer) {
                Ok(_) => {
                    buffer.append(&mut tmp);
                    self.get_len_type(&buffer);
                }
                Err(e) => self.error = Some(Box::new(e)),
            }
        }
        // TODO: handle parsing of the payload now that we know we have
        // Arrived here, we are supposed to have the whole frame
        None
    }
}
