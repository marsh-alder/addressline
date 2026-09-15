//! CLI wrapper around `addressline`: reads one address per line and prints
//! either the normalized form or a parse error, one line of output per line
//! of input. Reads from files given as arguments, or from stdin if no
//! arguments are given (or an argument is exactly `-`).

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::ExitCode;

use addressline::parse_line;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let sources: Vec<String> = if args.is_empty() { vec!["-".to_string()] } else { args };

    let mut had_error = false;

    for source in &sources {
        let reader: Box<dyn BufRead> = match source.as_str() {
            "-" => Box::new(BufReader::new(io::stdin())),
            path => match File::open(path) {
                Ok(f) => Box::new(BufReader::new(f)),
                Err(e) => {
                    eprintln!("addressline: {path}: {e}");
                    had_error = true;
                    continue;
                }
            },
        };

        let label = if source == "-" { "stdin" } else { source.as_str() };

        for (i, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("addressline: {label}:{}: {e}", i + 1);
                    had_error = true;
                    continue;
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            match parse_line(&line) {
                Ok(addr) => println!("{addr}"),
                Err(e) => {
                    eprintln!("addressline: {label}:{}: {e}", i + 1);
                    had_error = true;
                }
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
