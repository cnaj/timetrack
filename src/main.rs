#![allow(dead_code)]

use std::io::BufRead;
use std::ops::Sub;
use std::time::Duration;
use std::{env, io};

use chrono::{DateTime, FixedOffset};

use std::fs::File;
use timetrack::fileread::{read_log_lines, DayCollector, LogLine, LogEntries};
use timetrack::taskregistry::{TaskRegistry, TaskRegistryIterator};
use timetrack::timelog::{LogEvent, TimelogEntry};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} LOG_FILE_PATH", args[0]);
        std::process::exit(1);
    }

    print_summaries(args[1].as_str())?;
    Ok(())
}

fn print_lines(path: &str) -> Result<(), String> {
    let lines = read_log_lines(path).map_err(|err| format!("Could not read file: {}", err))?;

    let mut last_entry: Option<TimelogEntry> = None;
    for line in lines.enumerate() {
        match line {
            (_, Ok(entry)) => match &entry {
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
            },
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

fn print_summaries(path: &str) -> Result<(), String> {
    let file = File::open(path).map_err(|err| format!("Could not read file: {}", err))?;
    let lines = io::BufReader::new(file).lines();
    let entries = LogEntries::new(lines);
    let task_registries = TaskRegistryIterator::new(entries);

    for registry in task_registries {
        let registry = registry?;
        print_day_summary(&registry)?;
    }

    Ok(())
}

fn print_day_summary(registry: &TaskRegistry) -> Result<(), String> {
    println!("=== {:?}", registry.get_start_time()?);
    for task in registry.get_tasks() {
        println!("{}", task);
    }

    let mut work_time = Duration::from_secs(0);
    for task in registry.get_tasks().iter().skip(1) {
        // skip Pause task
        work_time += task.duration;
    }
    println!();

    println!(
        "-- Work time: {}",
        format_duration(&registry.get_work_duration())
    );
    println!();

    println!("-- Work hours:");
    println!("on   \toff  \ttime \tpause");
    let mut last_off: Option<DateTime<FixedOffset>> = None;
    for (on, off) in registry.get_work_times() {
        let delta = format_duration(&off.sub(*on).to_std().unwrap());
        let pause = match last_off {
            Some(last_off) => format_duration(&on.sub(last_off).to_std().unwrap()),
            None => "".to_string(),
        };
        last_off = Some(*off);
        println!(
            "{}\t{}\t{}\t{}",
            on.format("%H:%M"),
            off.format("%H:%M"),
            delta,
            pause
        );
    }

    println!();
    Ok(())
}

fn format_duration(work_time: &Duration) -> String {
    let secs = work_time.as_secs();
    let mins = secs / 60;
    let m = mins % 60;
    let h = mins / 60;
    format!("{:02}:{:02}", h, m)
}
