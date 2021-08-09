#[cfg(test)]
mod test;

use fnv::FnvHashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct Base64 {
    char_array: [u8; 64],
    char_to_sixlet: FnvHashMap<u8, u8>,
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
        let mut char_array: [u8; 64] = [u8::default(); 64];
        let mut char_to_sixlet = FnvHashMap::default();

        for (i, c) in (0..=25).zip('A'..='Z') {
            char_array[i] = c as u8;
            char_to_sixlet.insert(c as u8, i as u8);
        }

        for (i, c) in (26..=51).zip('a'..='z') {
            char_array[i] = c as u8;
            char_to_sixlet.insert(c as u8, i as u8);
        }

        for (i, c) in (52..=61).zip('0'..='9') {
            char_array[i] = c as u8;
            char_to_sixlet.insert(c as u8, i as u8);
        }

        char_array[62] = '+' as u8;
        char_array[63] = '/' as u8;

        char_to_sixlet.insert('+' as u8, 62);
        char_to_sixlet.insert('/' as u8, 63);

        char_to_sixlet.insert('=' as u8, 0);

        Base64 {
            char_array,
            char_to_sixlet,
        }
    }

    pub fn encode(&self, buffer: &[u8]) -> Vec<u8> {
        let mut v = Vec::with_capacity(((buffer.len() / 3) + 1) * 4);

        let mut iter = buffer.iter();
        const BITMASK: u8 = 0b111111;

        let reminder = loop {
            let mut placeholder = match (iter.next(), iter.next(), iter.next()) {
                (Some(b1), Some(b2), Some(b3)) => u32::from_be_bytes([0, *b1, *b2, *b3]),
                v => break v,
            };

            for _ in 0..4 {
                placeholder <<= 6;
                v.push(self.char_array[((placeholder.to_be_bytes()[0]) & BITMASK) as usize]);
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
                v.push(self.char_array[((reminder.to_be_bytes()[0]) & BITMASK) as usize]);
            }

            for _ in bytes..4 {
                v.push('=' as u8);
            }
        }

        v
    }

    pub fn decode(&self, enc_buf: &[u8]) -> Result<Vec<u8>, DecodingError> {
        let mut v = Vec::with_capacity((enc_buf.len() / 4) * 3);

        let to_strip = if let Some(s) = enc_buf.rsplit(|&c| c != '=' as u8).next() {
            s.len()
        } else {
            0
        };

        let mut chars = enc_buf.iter();

        while let Some(_) = chars.clone().next() {
            let mut placeholder = 0;

            for c in chars.clone().take(4) {
                placeholder <<= 6;

                placeholder |= match self.char_to_sixlet.get(&c) {
                    Some(idx) => *idx as u32,
                    None => return Err(DecodingError),
                };

                chars.next();
            }

            v.extend_from_slice(&placeholder.to_be_bytes()[1..]);
        }

        v.truncate(v.len() - to_strip);

        Ok(v)
    }
}
