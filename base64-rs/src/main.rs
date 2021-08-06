use std::env;
use std::process;

use base64_rs::run;
use base64_rs::OperationMode;

enum ExitCode {
    Success = 0,
    Failure = 1,
}

fn main() {
    let mut operation_mode = OperationMode::Encode;
    let mut path = None;

    for arg in env::args().skip(1) {
        match &arg[..] {
            "-d" => operation_mode = OperationMode::Decode,
            arg => path = Some(String::from(arg)),
        }
    }

    process::exit(match run(path, operation_mode) {
        Ok(_) => ExitCode::Success as i32,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::Failure as i32
        }
    });
}
