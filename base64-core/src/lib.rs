#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct Base64 {
    char_array: [char; 64],
    char_to_sixlet: HashMap<char, u8>,
}

#[derive(Debug)]
pub struct DecodingError;

impl Display for DecodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Input being decoded contains invalid characters")
    }
}

impl Error for DecodingError {}

impl Base64 {
    pub fn new() -> Base64 {
        let mut char_array: [char; 64] = [char::default(); 64];
        let mut char_to_sixlet = HashMap::new();

        for (i, c) in (0..=25).zip('A'..='Z') {
            char_array[i] = c;
            char_to_sixlet.insert(c, i as u8);
        }

        for (i, c) in (26..=51).zip('a'..='z') {
            char_array[i] = c;
            char_to_sixlet.insert(c, i as u8);
        }

        for (i, c) in (52..=61).zip('0'..='9') {
            char_array[i] = c;
            char_to_sixlet.insert(c, i as u8);
        }

        char_array[62] = '+';
        char_array[63] = '/';

        char_to_sixlet.insert('+', 62);
        char_to_sixlet.insert('/', 63);

        char_to_sixlet.insert('=', 0);

        Base64 {
            char_array,
            char_to_sixlet,
        }
    }

    pub fn encode(&self, buffer: &[u8]) -> String {
        let mut s = String::with_capacity(((buffer.len() / 3) + 1) * 4);

        let mut iter = buffer.iter();
        const BITMASK: u8 = 0b111111;

        let reminder = loop {
            let mut placeholder = match (iter.next(), iter.next(), iter.next()) {
                (Some(b1), Some(b2), Some(b3)) => u32::from_be_bytes([0, *b1, *b2, *b3]),
                v => break v,
            };

            for _ in 0..4 {
                placeholder <<= 6;
                s.push(self.char_array[((placeholder.to_be_bytes()[0]) & BITMASK) as usize]);
            }

        };

        let reminder = match reminder {
            (Some(b1), Some(b2), None) => Some((u32::from_be_bytes([0, *b1, *b2, 0]), 3)),
            (Some(b1), None, None) => Some((u32::from_be_bytes([0, *b1, 0, 0]), 2)),
            _ => None,
        };

        if let Some((mut reminder, bytes)) = reminder {
            for _ in 0..bytes {
                reminder <<= 6;
                s.push(self.char_array[((reminder.to_be_bytes()[0]) & BITMASK) as usize]);
            }

            for _ in bytes..4 {
                s.push('=');
            }
        }

        return s;
    }

    pub fn decode(&self, enc_buf: &str) -> Result<Vec<u8>, DecodingError> {
        let mut v = Vec::with_capacity((enc_buf.len() / 4) * 3);

        let to_strip = enc_buf.len() - enc_buf.trim_end_matches('=').len();

        let mut chars = enc_buf.bytes();

        loop {
            let mut placeholder = 0;

            for c in chars.clone().take(4) {
                placeholder <<= 6;

                placeholder |= match self.char_to_sixlet.get(&(c as char)) {
                    Some(idx) => *idx as u32,
                    None => return Err(DecodingError),
                };

                chars.next();
            }

            v.extend_from_slice(&placeholder.to_be_bytes()[1..]);

            if let None = chars.clone().next() {
                break;
            }
        }

        v.truncate(v.len() - to_strip);

        Ok(v)
    }
}
