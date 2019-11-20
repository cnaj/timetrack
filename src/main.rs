#![allow(dead_code)]

pub mod fileread;
pub mod timelog;

use crate::fileread::{read_log_lines, LogLine, DayCollector};
use crate::timelog::{LogEvent, TimelogEntry};
use std::env;
use std::ops::Sub;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} LOG_FILE_PATH", args[0]);
        std::process::exit(1);
    }

    gather_days(args[1].as_str())?;
    Ok(())
}

fn print_lines(path: &str) -> Result<(), String> {
    let lines = read_log_lines(path).map_err(|err| format!("Could not read file: {}", err))?;

    let mut last_entry: Option<TimelogEntry> = None;
    for line in lines {
        match line {
            Ok(entry) => {
                match entry {
                    LogLine::Entry(entry) => {
                        if let LogEvent::On = entry.event {
                            last_entry = None;
                        }

                        match last_entry {
                            Some(last_entry) => {
                                let diff = entry.time.sub(last_entry.time);
                                println!("{:?};    {} minutes", last_entry, diff.num_minutes());
                            }
                            None => println!("Start of new day"),
                        }
                        last_entry = Some(entry);
                    }
                    LogLine::Ignored(_) => println!(),
                }
            }
            Err(err) => println!("Error: {}", err),
        }
    }
    Ok(())
}

fn gather_tasks(path: &str) -> Result<(), String> {
    let lines = read_log_lines(path).map_err(|err| format!("Could not read file: {}", err))?;

    let lines = lines.filter_map(|res| match res {
        Ok(line) => match line {
            LogLine::Entry(entry) => Some(Ok(entry)),
            LogLine::Ignored(_) => None,
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

fn gather_days(path: &str) -> Result<(), String> {
    let lines = read_log_lines(path).map_err(|err| format!("Could not read file: {}", err))?;

    let day_collector = DayCollector::new(lines);

    for day in day_collector {
        let entries = day?;
        println!("{:?}", entries);
    }

    Ok(())
}
