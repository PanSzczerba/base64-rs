use std::error::Error;

use std::io;
use std::io::Read;
use std::io::Write;

use std::fs::File;

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

fn read_process_write<R, W, P>(
    mut reader: R,
    mut writer: W,
    processor: P,
) -> Result<(), Box<dyn Error>>
where
    R: Read,
    W: Write,
    P: Fn(&[u8]) -> Result<Vec<u8>, Box<dyn Error>>,
{
    const BUFFER_SIZE: usize = 1023 * 1024;

    let mut buffer = vec![0; BUFFER_SIZE];

    loop {
        let read = reader.read_exact_or_eof(&mut buffer[..])?;
        let proc_vec = processor(&buffer[..read])?;
        writer.write_all(&proc_vec[..])?;

        if read < buffer.len() {
            break;
        }
    }

    writer.flush()?;

    Ok(())
}

pub fn run(path: Option<String>, operation_mode: OperationMode) -> Result<(), Box<dyn Error>> {
    let reader: Box<dyn Read> = match path {
        Some(path) if path != "-" => Box::new(File::open(path)?),
        _ => Box::new(io::stdin()),
    };
    let writer = io::stdout();
    let base64 = Base64::new();

    let result = match operation_mode {
        OperationMode::Encode => {
            read_process_write(reader, writer, |buffer| Ok(base64.encode(buffer)))
        }
        OperationMode::Decode => read_process_write(reader, writer, |buffer| {
            let trailing_whitespace = buffer
                .iter()
                .rev()
                .take_while(|&c| c.is_ascii_whitespace())
                .count();
            let buffer = &buffer[..buffer.len() - trailing_whitespace];

            base64.decode(buffer).map_err(Box::<dyn Error>::from)
        }),
    };

    result.or_else(|e| match e.downcast_ref::<io::Error>() {
        Some(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        _ => Err(e),
    })
}
