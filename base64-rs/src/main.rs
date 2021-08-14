use std::env;
use std::process;

use base64_rs::run;
use base64_rs::OperationMode;

enum ExitCode {
    Success = 0,
    Failure = 1,
}

fn print_usage() {
    eprintln!(
        "Usage: base64-rs [OPTIONS] [FILE]
Small command line utility for encoding/decoding data with base64.
If FILE is not supplied or is set to \"-\" the input will be taken
directly from stdin.

Options:
  -d                     If this flag is set, the input will be
                         decoded from base64.
  --help                 Displays this message."
    );
}

fn main() {
    let mut operation_mode = OperationMode::Encode;
    let mut path = None;

    for arg in env::args().skip(1) {
        match &arg[..] {
            "-d" => operation_mode = OperationMode::Decode,
            "--help" => {
                print_usage();
                return;
            }
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
