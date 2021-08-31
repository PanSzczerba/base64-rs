use std::error::Error;

use std::io;
use std::io::Read;
use std::io::Write;

use std::fs::File;
use std::hint;
use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

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
    R: Read + Send,
    W: Write + Send + 'static,
    P: Fn(&[u8]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>,
    P: Send + 'static,
{
    const BUFFER_SIZE: usize = 1023 * 1024;
    const MAX_BUFFERS: i8 = 16;
    let buf_cnt = Arc::new(AtomicI8::new(0));
    let reader_buf_cnt = Arc::clone(&buf_cnt);

    let (rtx, rrx): (Sender<(Vec<u8>, usize)>, Receiver<(Vec<u8>, usize)>) = mpsc::channel();
    let (wtx, wrx) = mpsc::channel();

    let process_thread: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> =
        thread::spawn(move || {
            for (buffer, cnt) in rrx.iter() {
                let proc_vec = processor(&buffer[..cnt])?;
                reader_buf_cnt.fetch_sub(1, Ordering::SeqCst);
                wtx.send(proc_vec)?;
            }
            Ok(())
        });

    let write_thread: JoinHandle<Result<(), io::Error>> = thread::spawn(move || {
        for proc_vec in wrx.iter() {
            writer.write_all(&proc_vec[..])?;
        }
        writer.flush()?;
        Ok(())
    });

    loop {
        if buf_cnt.load(Ordering::SeqCst) < MAX_BUFFERS {
            let mut buffer = Vec::<u8>::with_capacity(BUFFER_SIZE);
            buffer.resize(BUFFER_SIZE, 0);

            buf_cnt.fetch_add(1, Ordering::SeqCst);

            let read = reader.read_exact_or_eof(&mut buffer[..])?;

            rtx.send((buffer, read))?;

            if read < BUFFER_SIZE {
                break;
            }
        } else {
            hint::spin_loop();
        }
    }
    drop(rtx);

    match process_thread
        .join()
        .expect("Couldn't join on processor thread")
    {
        Ok(_) => (),
        Err(e) => return Err(e),
    }
    write_thread
        .join()
        .expect("Couldn't join on writer thread")?;
    Ok(())
}

pub fn run(path: Option<String>, operation_mode: OperationMode) -> Result<(), Box<dyn Error>> {
    let reader: Box<dyn Read + Send + Sync> = match path {
        Some(path) if path != "-" => Box::new(File::open(path)?),
        _ => Box::new(io::stdin()),
    };
    let writer = io::stdout();
    let base64 = Base64::new();

    let result = match operation_mode {
        OperationMode::Encode => {
            read_process_write(reader, writer, move |buffer| Ok(base64.encode(buffer)))
        }
        OperationMode::Decode => read_process_write(reader, writer, move |buffer| {
            let trailing_whitespace = buffer
                .iter()
                .rev()
                .take_while(|&c| c.is_ascii_whitespace())
                .count();
            let buffer = &buffer[..buffer.len() - trailing_whitespace];

            base64
                .decode(buffer)
                .or_else(|e| Err(Box::<dyn Error + Send + Sync>::from(e)))
        }),
    };

    result.or_else(|e| match e.downcast_ref::<io::Error>() {
        Some(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        _ => Err(e),
    })
}
