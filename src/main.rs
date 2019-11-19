#![allow(dead_code)]

use crate::fileread::{read_log_lines, LogLine};
use std::env;

pub mod fileread;
pub mod timelog;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} LOG_FILE_PATH", args[0]);
        std::process::exit(1);
    }

    gather_tasks(args[1].as_str())?;
    Ok(())
}

fn print_lines(path: &str) -> Result<(), String> {
    let lines = read_log_lines(path)
        .map_err(|err| format!("Could not read file: {}", err))?;

    for line in lines {
        match line {
            Ok(entry) => println!("{:?}", entry),
            Err(err) => println!("Error: {}", err),
        }
    }
    Ok(())
}

fn gather_tasks(path: &str) -> Result<(), String> {
    let lines = read_log_lines(path)
        .map_err(|err| format!("Could not read file: {}", err))?;

    let lines = lines
        .filter_map(|res| match res {
            Ok(line) => {
                match line {
                    LogLine::Entry(entry) => Some(Ok(entry)),
                    LogLine::Ignored => None,
                }
            },
            Err(err) => Some(Err(err)),
        });

    for line in lines {
        match line {
            Ok(entry) => println!("{:?}", entry),
            Err(err) => println!("Error: {}", err),
        }
    }

    Ok(())
}
