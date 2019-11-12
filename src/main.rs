use crate::fileread::read_log_lines;
use std::env;

pub mod fileread;
pub mod timelog;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: timetrack LOG_FILE_PATH");
        std::process::exit(1);
    }

    match read_log_lines(args[1].as_str()) {
        Err(err) => println!("ERROR: Could not read file: {}", err),
        Ok(lines) => {
            for line in lines.into_iter() {
                match line {
                    Ok(entry) => println!("{:?}", entry),
                    Err(err) => println!("ERROR: {}", err),
                }
            };
        }
    }
}
