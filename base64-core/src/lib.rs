#[cfg(test)]
mod test;

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct Base64 {
    char_array: [u8; 64],
    char_to_sixlet: [u8; 128],
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
        let mut char_array: [u8; 64] = [0; 64];
        let mut char_to_sixlet = [u8::MAX; 128];

        for (i, c) in (0..=25)
            .zip('A'..='Z')
            .chain((26..=51).zip('a'..='z'))
            .chain((52..=61).zip('0'..='9'))
            .chain((62..=63).zip(vec!['+', '/'].into_iter()))
        {
            char_array[i] = c as u8;
            char_to_sixlet[c as usize] = i as u8;
        }

        char_to_sixlet['=' as usize] = 0;

        Base64 {
            char_array,
            char_to_sixlet,
        }
    }

    pub fn encode(&self, buffer: &[u8]) -> Vec<u8> {
        const BITMASK: u8 = 0b111111;

        let mut v = Vec::with_capacity(((buffer.len() / 3) + 1) * 4);

        let (buffer, reminder) = buffer.split_at(buffer.len() - buffer.len() % 3);
        let mut bytes = buffer.iter();

        while let Some(_) = bytes.clone().next() {
            let (&b1, &b2, &b3) = (
                bytes.next().unwrap(),
                bytes.next().unwrap(),
                bytes.next().unwrap(),
            );

            v.extend_from_slice(
                &[
                    self.char_array[(b1 >> 2) as usize],
                    self.char_array[(((b1 << 4) | (b2 >> 4)) & BITMASK) as usize],
                    self.char_array[(((b2 << 2) | (b3 >> 6)) & BITMASK) as usize],
                    self.char_array[(b3 & BITMASK) as usize],
                ][..],
            );
        }

        let reminder = match reminder {
            [b1, b2] => Some((u32::from_be_bytes([0, *b1, *b2, 0]), 3)),
            [b1] => Some((u32::from_be_bytes([0, *b1, 0, 0]), 2)),
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

            for &c in chars.clone().take(4) {
                placeholder <<= 6;

                placeholder |= match self.char_to_sixlet.get(c as usize) {
                    None => return Err(DecodingError),
                    Some(&u8::MAX) => return Err(DecodingError),
                    Some(&n) => n as u32,
                };

                chars.next();
            }

            v.extend_from_slice(&placeholder.to_be_bytes()[1..]);
        }

        v.truncate(v.len() - to_strip);

        Ok(v)
    }
}
