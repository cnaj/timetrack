#![allow(dead_code)]

use std::env;
use std::ops::Sub;
use std::time::Duration;

use chrono::{DateTime, FixedOffset};

use crate::fileread::{DayCollector, LogLine, read_log_lines};
use crate::taskregistry::TaskRegistry;
use crate::timelog::{LogEvent, TimelogEntry};

pub mod fileread;
pub mod taskregistry;
pub mod timelog;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} LOG_FILE_PATH", args[0]);
        std::process::exit(1);
    }

    gather_day_tasks(args[1].as_str())?;
    Ok(())
}

fn print_lines(path: &str) -> Result<(), String> {
    let lines = read_log_lines(path).map_err(|err| format!("Could not read file: {}", err))?;

    let mut last_entry: Option<TimelogEntry> = None;
    for line in lines.enumerate() {
        match line {
            (_, Ok(entry)) => {
                match &entry {
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
                        last_entry = Some(entry.clone());
                    }
                    LogLine::Ignored(_) => println!(),
                }
            }
            (n, Err(err)) => println!("Error (line {}): {}", n, err),
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


fn gather_day_tasks(path: &str) -> Result<(), String> {
    let lines = read_log_lines(path).map_err(|err| format!("Could not read file: {}", err))?;

    let day_collector = DayCollector::new(lines);

    for day in day_collector {
        let entries = day?;
        match entries.start {
            Some(start) => {
                let it = entries.lines.iter()
                    .filter_map(|line| match &line.1 {
                        LogLine::Entry(entry) => Some((line.0, entry.clone())),
                        LogLine::Ignored(_) => None,
                    });

                let registry = TaskRegistry::build(it)?;
                println!("=== {:?}", start);
                for task in registry.get_tasks() {
                    println!("{}", task);
                }

                let mut work_time = Duration::from_secs(0);
                for task in registry.get_tasks().iter().skip(1) { // skip Pause task
                    work_time += task.duration;
                }
                println!();

                println!("-- Work time: {}", format_duration(&mut work_time));
                println!();

                println!("-- Work hours:");
                println!("on   \toff  \ttime \tpause");
                let mut last_off: Option<DateTime<FixedOffset>> = None;
                for (on, off) in registry.get_work_times() {
                    let delta = format_duration(&off.sub(*on).to_std().unwrap());
                    let pause = match last_off {
                        Some(last_off) => format_duration(&on.sub(last_off).to_std().unwrap()),
                        None => "".to_string()
                    };
                    last_off = Some(*off);
                    println!("{}\t{}\t{}\t{}", on.format("%H:%M"), off.format("%H:%M"), delta, pause);
                }

                println!();
            }
            None => {}
        }
    }

    Ok(())
}

fn format_duration(work_time: &Duration) -> String {
    let secs = work_time.as_secs();
    let mins = secs / 60;
    let m = mins % 60;
    let h = mins / 60;
    format!("{:02}:{:02}", h, m)
}
