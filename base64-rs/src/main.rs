use std::env;
use std::process;

use base64_rs::run;
use base64_rs::OperationMode;

enum ExitCode {
    Success = 0,
    Failure = 1,
}

fn print_usage() {
    println!("Usage: base64-rs [-d] [FILE]");
    println!("Small command line util for encoding/decoding data with base64.");
    println!("If FILE is not supplied or is set to \"-\" it will take input");
    println!("directly from stdin");
    println!("");
    println!("Arguments:");
    println!("   -d                     If this flag is set, the input will be");
    println!("                          decoded from base64.");
}

fn main() {
    let mut operation_mode = OperationMode::Encode;
    let mut path = None;

    for arg in env::args().skip(1) {
        match &arg[..] {
            "-d" => operation_mode = OperationMode::Decode,
            "--help" => { print_usage(); return; }
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
