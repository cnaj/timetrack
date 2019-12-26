use std::io;
use std::fs::File;
use std::io::BufRead;
use std::ops::Sub;
use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use clap::{App, Arg};

use timetrack::fileread::{DayCollector, LogLines};
use timetrack::taskregistry::TaskRegistry;

fn main() -> Result<(), String> {
    let matches = App::new("timetrack")
        .about("Command-line time tracking tool")
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("Path to input file")
            .required(true)
            .takes_value(true)
        )
        .get_matches();

    let file_path = matches.value_of("file").unwrap();

    print_summaries(file_path)?;
    Ok(())
}

fn print_summaries(path: &str) -> Result<(), String> {
    let file = File::open(path).map_err(|err| format!("Could not read file {:?}: {}", path, err))?;
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

    println!(
        "-- Work time: {}",
        format_duration(&registry.get_work_duration())
    );

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
