use std::{env, io};
use std::fs::File;
use std::io::BufRead;
use std::ops::Sub;
use std::time::Duration;

use chrono::{DateTime, FixedOffset};

use timetrack::fileread::{DayCollector, LogLines};
use timetrack::taskregistry::TaskRegistry;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} LOG_FILE_PATH", args[0]);
        std::process::exit(1);
    }

    print_summaries(args[1].as_str())?;
    Ok(())
}

fn print_summaries(path: &str) -> Result<(), String> {
    let file = File::open(path).map_err(|err| format!("Could not read file: {}", err))?;
    let lines = io::BufReader::new(file).lines();
    let lines = LogLines::new(lines);
    let day_collector = DayCollector::new(lines);

    for day in day_collector {
        let day = day?;
        if let Some(tasks) = day.tasks {
            print_day_summary(&tasks)?;
        }
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
