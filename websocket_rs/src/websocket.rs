use std::error::Error;
use std::io::Read;
use std::net::TcpStream;

pub struct WebSocket {
    socket: TcpStream,
}

impl WebSocket {
    pub fn new(socket: TcpStream) -> Self {
        Self { socket }
    }

    pub fn read_frame(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buffer = [0u8; 10_000];
        let n = self.socket.read(&mut buffer)?;

        if n > 6 {
            let payload = get_payload(&buffer);
            Ok(payload)
        } else {
            Ok("".to_string())
        }
    }
}

fn get_payload(buffer: &[u8]) -> String {
    let len = get_len(&buffer);
    let mask = get_mask(&buffer);
    let payload = extract_payload(&buffer, len);
    unmask_payload(payload, mask)
}

fn get_len(buffer: &[u8]) -> usize {
    // We mask the first byte as it is not part of the length
    let size_byte = 0b01111111 & buffer[1];
    match size_byte {
        126 => ((buffer[2] as usize) << 8) + buffer[3] as usize,
        127 => {
            ((buffer[4] as usize) << (8 * 5))
                + ((buffer[5] as usize) << (8 * 4))
                + ((buffer[6] as usize) << (8 * 3))
                + ((buffer[7] as usize) << (8 * 2))
                + ((buffer[8] as usize) << (8 * 1))
                + (buffer[9] as usize)
        }
        _ => size_byte as usize,
    }
}

fn get_mask(buffer: &[u8]) -> &[u8] {
    let byte_size = 0b0111_1111 & buffer[1];
    match byte_size {
        126 => &buffer[4..8],
        127 => &buffer[10..14],
        _ => &buffer[2..6],
    }
}

fn extract_payload(buffer: &[u8], len: usize) -> &[u8] {
    let byte_size = 0b0111_1111 & buffer[1];
    let start = match byte_size {
        126 => 2 + 2 + 6 + 4,
        127 => 2 + 2 + 4,
        _ => 2 + 4,
    };
    &buffer[start..(start + len)]
}

fn unmask_payload(payload: &[u8], mask: &[u8]) -> String {
    let mut unmasked_payload: Vec<u8> = Vec::new();
    for (i, char) in payload.into_iter().enumerate() {
        let j = i % 4;
        unmasked_payload.push(*char ^ mask[j]);
    }
    String::from_utf8_lossy(&unmasked_payload).to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parse_basic_len_correctly() {
        // This frame carry: Hello
        let buffer = [129, 133, 166, 51, 46, 40, 238, 86, 66, 68, 201];
        let len = get_len(&buffer);
        assert_eq!(len, 5)
    }

    #[test]
    fn it_parse_basic_len_correctly_double_size() {
        // The following string is the payload. Length = 128
        // Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque pe
        let buffer = [
            129, 254, 0, 128, 234, 188, 150, 161, 166, 211, 228, 196, 135, 156, 255, 209, 153, 201,
            251, 129, 142, 211, 250, 206, 152, 156, 229, 200, 158, 156, 247, 204, 143, 200, 186,
            129, 137, 211, 248, 210, 143, 223, 226, 196, 158, 201, 243, 211, 202, 221, 242, 200,
            154, 213, 229, 194, 131, 210, 241, 129, 143, 208, 255, 213, 196, 156, 215, 196, 132,
            217, 247, 207, 202, 223, 249, 204, 135, 211, 242, 206, 202, 208, 255, 198, 159, 208,
            247, 129, 143, 219, 243, 213, 202, 216, 249, 205, 133, 206, 184, 129, 171, 217, 248,
            196, 139, 210, 182, 204, 139, 207, 229, 192, 196, 156, 213, 212, 135, 156, 229, 206,
            137, 213, 255, 210, 202, 210, 247, 213, 133, 205, 227, 196, 202, 204, 247, 202,
        ];

        let len = get_len(&buffer);
        assert_eq!(len, 128)
    }

    #[test]
    fn it_parse_basic_len_correctly_double_size_large() {
        // The following string is the payload. Length = 35000 sent by postman.
        let buffer = [129, 254, 136, 184, 10, 8, 221, 131, 70, 103, 175, 230];
        let expected = 0b10001000_00000000 + 0b10111000;
        let len = get_len(&buffer);
        assert_eq!(len, expected);
        assert_eq!(len, 35000);
    }

    #[test]
    fn it_parse_basic_len_correctly_big_size() {
        let buffer = [129, 255, 0, 0, 0, 0, 0, 1, 136, 184, 175, 230];
        let expected = 0b1_00000000_00000000 + 0b10001000_00000000 + 0b10111000;
        let len = get_len(&buffer);
        assert_eq!(len, expected);
    }

    #[test]
    fn it_get_mask() {
        let buffer = [129, 133, 166, 51, 46, 40, 238, 86, 66, 68, 201];
        let expected = [166, 51, 46, 40];
        let mask = get_mask(&buffer);
        test_mask(mask, &expected)
    }
    fn test_mask(mask: &[u8], expected: &[u8]) {
        for (i, value) in expected.iter().enumerate() {
            assert_eq!(*value, mask[i]);
        }
    }

    #[test]
    fn it_get_mask_double() {
        let buffer = [129, 254, 166, 51, 46, 40, 238, 86, 66, 68, 201];
        let expected = [46, 40, 238, 86];
        let mask = get_mask(&buffer);
        test_mask(mask, &expected)
    }

    #[test]
    fn it_get_mask_big() {
        let buffer = [129, 255, 0, 0, 46, 40, 238, 86, 66, 68, 201, 100, 20, 32];
        let expected = [201, 100, 20, 32];
        let mask = get_mask(&buffer);
        test_mask(mask, &expected)
    }

    #[test]
    fn it_parse_a_basic_message() {
        // This frame carry: Hello
        let buffer = [129, 133, 166, 51, 46, 40, 238, 86, 66, 68, 201];
        let payload = get_payload(&buffer);
        assert_eq!(payload.as_str(), "Helloo")
    }
}
