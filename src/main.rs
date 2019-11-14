#![allow(dead_code)]

use crate::fileread::read_log_lines;
use std::env;

pub mod fileread;
pub mod timelog;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: timetrack LOG_FILE_PATH");
        std::process::exit(1);
    }

    print_lines(args[1].as_str())?;
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

    // TODO

    Ok(())
}
