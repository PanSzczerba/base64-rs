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

        loop {
            let (mut placeholder, chars) = match (iter.next(), iter.next(), iter.next()) {
                (Some(b1), Some(b2), Some(b3)) => (u32::from_be_bytes([0, *b1, *b2, *b3]), 4),
                (Some(b1), Some(b2), None) => (u32::from_be_bytes([0, *b1, *b2, 0]), 3),
                (Some(b1), None, None) => (u32::from_be_bytes([0, *b1, 0, 0]), 2),
                _ => break,
            };

            for _ in 0..chars {
                placeholder <<= 6;
                s.push(self.char_array[((placeholder.to_be_bytes()[0]) & BITMASK) as usize]);
            }

            for _ in chars..4 {
                s.push('=');
            }
        }

        return s;
    }

    pub fn decode(&self, enc_buf: &str) -> Result<Vec<u8>, DecodingError> {
        let mut v = Vec::with_capacity((enc_buf.len() / 4) * 3);

        let mut placeholder: u32 = 0;
        let mut cnt = 0;

        let to_strip = enc_buf.len() - enc_buf.trim_end_matches('=').len();

        for c in enc_buf.chars() {
            placeholder <<= 6;

            placeholder |= match self.char_to_sixlet.get(&c) {
                Some(idx) => *idx as u32,
                None => return Err(DecodingError),
            };
            cnt += 1;

            if cnt == 4 {
                v.extend_from_slice(&placeholder.to_be_bytes()[1..]);

                placeholder = 0;
                cnt = 0;
            }
        }

        v.truncate(v.len() - to_strip);

        Ok(v)
    }
}
