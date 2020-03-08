use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::ops::Sub;
use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use clap::{App, Arg, ArgMatches, SubCommand};

use timetrack::fileread::{DayCollector, LogLines};
use timetrack::taskregistry::TaskRegistry;

enum SummaryScope {
    All,
    Last(usize),
}

fn main() -> Result<(), String> {
    let matches = App::new("timetrack")
        .about("Command-line time tracking tool")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Path to input file")
                .required(true)
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("summary")
                .about("Displays a task and time summary per work day.")
                .subcommand(
                    SubCommand::with_name("all").about("Displays tasks for all available days"),
                )
                .subcommand(
                    SubCommand::with_name("last")
                        .about("Displays tasks of the last days")
                        .arg(Arg::with_name("number").default_value("1")),
                ),
        )
        .subcommand(
            SubCommand::with_name("last-active").about("Displays the last recorded active task"),
        )
        .get_matches();

    let file_path = matches.value_of("file").unwrap();

    match matches.subcommand() {
        ("summary", Some(sub_matches)) => cmd_summary(sub_matches, file_path)?,
        ("last-active", Some(_)) => cmd_last_active(file_path)?,
        _ => print_summaries(file_path, SummaryScope::Last(1))?,
    };

    Ok(())
}

fn cmd_summary(matches: &ArgMatches, file_path: &str) -> Result<(), String> {
    let scope = match matches.subcommand() {
        ("all", Some(_)) => SummaryScope::All,
        ("last", Some(last_matches)) => match last_matches.value_of("number") {
            None => SummaryScope::Last(1),
            Some(number) => match number.parse::<usize>() {
                Ok(n) => SummaryScope::Last(n),
                Err(e) => return Err(format!("Invalid number given: {}", e)),
            },
        },
        _ => SummaryScope::Last(1),
    };

    print_summaries(file_path, scope)
}

fn cmd_last_active(path: &str) -> Result<(), String> {
    let file =
        File::open(path).map_err(|err| format!("Could not read file {:?}: {}", path, err))?;
    let lines = io::BufReader::new(file).lines();
    let lines = LogLines::new(lines);
    let day_collector = DayCollector::new(lines);

    let last_active = match day_collector.last() {
        Some(day_result) => {
            let tasks = day_result?.tasks;
            tasks.get_last_active()
        }
        None => None,
    };

    if last_active.is_some() {
        println!("{}", last_active.unwrap().name);
    }

    Ok(())
}

fn print_summaries(path: &str, scope: SummaryScope) -> Result<(), String> {
    let file =
        File::open(path).map_err(|err| format!("Could not read file {:?}: {}", path, err))?;
    let lines = io::BufReader::new(file).lines();
    let lines = LogLines::new(lines);
    let day_collector = DayCollector::new(lines);

    match scope {
        SummaryScope::All => {
            for day in day_collector {
                let day = day?;
                print_day_summary(&day.tasks)?;
                println!();
            }
        }
        SummaryScope::Last(n) => {
            let mut day_tasks = VecDeque::with_capacity(n);

            for day in day_collector {
                let day = day?;
                if day_tasks.len() == n {
                    day_tasks.pop_front();
                }
                day_tasks.push_back(day.tasks);
            }

            for tasks in day_tasks {
                print_day_summary(&tasks)?;
                println!();
            }
        }
    };

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

    Ok(())
}

fn format_duration(work_time: &Duration) -> String {
    let secs = work_time.as_secs();
    let mins = secs / 60;
    let m = mins % 60;
    let h = mins / 60;
    format!("{:02}:{:02}", h, m)
}
