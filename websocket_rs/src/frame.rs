use std::error;
use std::io::Read;
use std::net::TcpStream;

#[derive(PartialEq, Debug)]
enum Opcode {
    Text,
    None,
}

#[derive(PartialEq, Debug)]
enum LenType {
    Single,
    Double,
    Big,
    None,
}
pub struct FrameHandler {
    socket: TcpStream,
    len_type: LenType,
    len: usize,
}

impl FrameHandler {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            len: 1024,
            len_type: LenType::None,
        }
    }

    pub fn get_next_frame(&mut self) -> Result<Option<String>, Box<dyn error::Error>> {
        // TODO: see if we need to handle a potential remaining from the last next
        let mut buffer: Vec<u8> = Vec::new();
        while buffer.len() <= (self.len + self.header_len()) && self.len_type == LenType::None {
            let mut tmp = vec![0u8; 1024];
            let n = self.socket.read(&mut tmp)?;
            if n == 0 {
                return Ok(None);
            }
            buffer.extend_from_slice(&tmp[..n]);
            self.set_payload_length(&buffer);
        }
        if buffer.len() != 0 {
            self.get_payload(&buffer)
        } else {
            Ok(None)
        }
    }

    fn set_payload_length(&mut self, buffer: &Vec<u8>) {
        if buffer.len() > 2 && self.len_type == LenType::None {
            let len_byte = buffer[1] & 0b01111111; // first byte is for mask bit
            self.len_type = match len_byte {
                126 => LenType::Double,
                127 => LenType::Big,
                _ => LenType::Single,
            };
            match self.len_type {
                LenType::None => {}
                LenType::Single => self.len = len_byte as usize,
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

    fn get_payload(&self, buffer: &[u8]) -> Result<Option<String>, Box<dyn error::Error>> {
        let opcode = get_opcode(buffer[0]);
        let mask = self.get_mask(buffer);
        let payload_buffer = self.get_payload_buffer(buffer);
        let unmasked_payload = self.unmask_payload(payload_buffer, mask);
        match opcode {
            Opcode::Text => Ok(Some(String::from_utf8_lossy(&unmasked_payload).to_string())),
            Opcode::None => panic!("Error: Frame opcode is none"),
        }
    }

    fn get_mask<'a>(&self, buffer: &'a [u8]) -> &'a [u8] {
        let offset: usize = 1 + self.get_len_offset();
        let end = offset + 4;
        &buffer[offset..end]
    }

    fn get_len_offset(&self) -> usize {
        match self.len_type {
            LenType::Single => 1,
            LenType::Double => 2,
            LenType::Big => 6,
            LenType::None => 0,
        }
    }

    fn get_payload_buffer<'a>(&self, buffer: &'a [u8]) -> &'a [u8] {
        let offset = 1 + self.get_len_offset() + 4;
        &buffer[offset..]
    }

    fn unmask_payload(&self, payload: &[u8], mask: &[u8]) -> Vec<u8> {
        let mut retval = vec![0u8; self.len];
        for i in 0..self.len {
            let j = i.wrapping_rem(4);
            retval[i] = payload[i] ^ mask[j]
        }
        retval
    }
}

fn get_opcode(byte: u8) -> Opcode {
    match byte & 0b00001111 {
        0b1 => Opcode::Text,
        _ => Opcode::None,
    }
}
