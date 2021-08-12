#[cfg(test)]
mod test;

use crate::DecodingErrorKind::{InvalidCharacter, InvalidLength};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct Base64 {
    char_array: [u8; 64],
    char_to_sixlet: [u8; 128],
}

#[derive(Debug, Copy, Clone)]
pub enum DecodingErrorKind {
    InvalidCharacter,
    InvalidLength,
}

#[derive(Debug)]
pub struct DecodingError {
    kind: DecodingErrorKind,
}

impl DecodingError {
    fn kind(&self) -> DecodingErrorKind {
        return self.kind;
    }
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match &self.kind {
            InvalidCharacter => write!(f, "Input being decoded contains invalid characters"),
            InvalidLength => write!(f, "Input string isn't properly aligned"),
        }
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

        while let (Some(&b1), Some(&b2), Some(&b3)) = (bytes.next(), bytes.next(), bytes.next()) {
            v.extend_from_slice(
                &[
                    self.char_array[(b1 >> 2) as usize],
                    self.char_array[(((b1 << 4) | (b2 >> 4)) & BITMASK) as usize],
                    self.char_array[(((b2 << 2) | (b3 >> 6)) & BITMASK) as usize],
                    self.char_array[(b3 & BITMASK) as usize],
                ][..],
            );
        }

        match reminder {
            [b1, b2] => v.extend_from_slice(
                &[
                    self.char_array[(b1 >> 2) as usize],
                    self.char_array[(((b1 << 4) | (b2 >> 4)) & BITMASK) as usize],
                    self.char_array[((b2 << 2) & BITMASK) as usize],
                    '=' as u8,
                ][..],
            ),
            [b1] => v.extend_from_slice(
                &[
                    self.char_array[(b1 >> 2) as usize],
                    self.char_array[((b1 << 4) & BITMASK) as usize],
                    '=' as u8,
                    '=' as u8,
                ][..],
            ),
            _ => (),
        };

        v
    }

    pub fn decode(&self, enc_buf: &[u8]) -> Result<Vec<u8>, DecodingError> {
        if enc_buf.len() % 4 != 0 {
            return Err(DecodingError {
                kind: InvalidLength,
            });
        }

        let mut v = Vec::with_capacity((enc_buf.len() / 4) * 3);

        let mut chars = enc_buf.iter();

        while let Some(_) = chars.clone().next() {
            let mut placeholder = 0;

            for &c in chars.clone().take(4) {
                placeholder <<= 6;

                placeholder |= match self.char_to_sixlet.get(c as usize) {
                    Some(&n) if n != u8::MAX => n as u32,
                    _ => {
                        return Err(DecodingError {
                            kind: InvalidCharacter,
                        })
                    }
                };

                chars.next();
            }

            v.extend_from_slice(&placeholder.to_be_bytes()[1..]);
        }

        let to_strip = enc_buf
            .iter()
            .rev()
            .take_while(|&&c| c == '=' as u8)
            .count();
        v.truncate(v.len() - to_strip);

        Ok(v)
    }
}
