use std::error::Error;

use std::io;
use std::io::Read;
use std::io::Write;

use std::fs::File;
use std::str;

use base64_core::Base64;

pub enum OperationMode {
    Encode,
    Decode,
}

trait ReadExt {
    fn read_exact_or_eof(&mut self, buffer: &mut [u8]) -> io::Result<usize>;
}

impl<R: Read> ReadExt for R {
    fn read_exact_or_eof(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut total_read: usize = 0;
        loop {
            match self.read(&mut buffer[total_read..]) {
                Ok(0) => break,
                Ok(n) => total_read += n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(total_read)
    }
}

const BUFFER_SIZE: usize = 3 * 1024 * 1024;

fn read_encode<R: Read>(mut reader: R) -> io::Result<()> {
    let mut buffer = Vec::<u8>::with_capacity(BUFFER_SIZE);
    buffer.resize(BUFFER_SIZE, 0);

    let encoder = Base64::new();
    let stdout = io::stdout();

    loop {
        let read = reader.read_exact_or_eof(&mut buffer[..])?;

        stdout
            .lock()
            .write(encoder.encode(&buffer[..read]).as_bytes())?;

        if read < buffer.len() {
            break;
        }
    }

    Ok(())
}

fn read_decode<R: Read>(mut reader: R) -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::<u8>::with_capacity(BUFFER_SIZE);
    buffer.resize(BUFFER_SIZE, 0);

    let encoder = Base64::new();
    let stdout = io::stdout();

    loop {
        let read = reader.read_exact_or_eof(&mut buffer[..])?;

        let vec = encoder.decode(str::from_utf8(&buffer[..read]).unwrap())?;
        stdout.lock().write(&vec[..])?;

        if read < buffer.len() {
            break;
        }
    }

    stdout.lock().flush()?;

    Ok(())
}

pub fn run(path: Option<String>, operation_mode: OperationMode) -> Result<(), Box<dyn Error>> {
    let reader: Box<dyn Read> = if let Some(path) = path {
        if path != "-" {
            Box::new(File::open(path)?)
        } else {
            Box::new(io::stdin())
        }
    } else {
        Box::new(io::stdin())
    };

    match operation_mode {
        OperationMode::Encode => read_encode(reader)?,
        OperationMode::Decode => read_decode(reader)?,
    };

    Ok(())
}
