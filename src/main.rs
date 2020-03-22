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
            SubCommand::with_name("last-active").about("Displays the last recorded active task"),
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
        .subcommand(SubCommand::with_name("tasks").about("Displays a list of recorded tasks"))
        .get_matches();

    let file_path = matches.value_of("file").unwrap();

    let mut w = io::stdout();
    match matches.subcommand() {
        ("last-active", Some(_)) => cmd_last_active(&mut w, file_path)?,
        ("summary", Some(sub_matches)) => cmd_summary(&mut w, sub_matches, file_path)?,
        ("tasks", Some(_)) => cmd_tasks(&mut w, file_path)?,
        _ => print_summaries(&mut w, file_path, SummaryScope::Last(1))?,
    };

    Ok(())
}

fn cmd_summary(mut w: impl io::Write, matches: &ArgMatches, file_path: &str) -> Result<(), String> {
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

    print_summaries(&mut w, file_path, scope)
}

fn cmd_last_active(mut w: impl io::Write, path: &str) -> Result<(), String> {
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
        writeln!(&mut w, "{}", last_active.unwrap().name).map_err(map_io_err)?;
    }

    Ok(())
}

fn cmd_tasks(mut w: impl io::Write, path: &str) -> Result<(), String> {
    let file =
        File::open(path).map_err(|err| format!("Could not read file {:?}: {}", path, err))?;
    let lines = io::BufReader::new(file).lines();
    let lines = LogLines::new(lines);
    let day_collector = DayCollector::new(lines);

    match day_collector.last() {
        Some(day_result) => {
            let registry = day_result?.tasks;
            print_tasks(&mut w, &registry).map_err(map_io_err)?;
        }
        None => {}
    };

    Ok(())
}

fn print_tasks(mut w: impl io::Write, registry: &TaskRegistry) -> io::Result<()> {
    let tasks = registry.get_tasks();

    writeln!(&mut w, "#\ttime\ttask name")?;
    for (n, task) in tasks.iter().enumerate().skip(1) {
        writeln!(&mut w, "{}\t{}", n, task)?;
    }
    writeln!(
        &mut w,
        "\t{}\ttotal work time",
        format_duration(&registry.get_work_duration())
    )?;
    Ok(())
}

fn print_summaries(mut w: impl io::Write, path: &str, scope: SummaryScope) -> Result<(), String> {
    let file =
        File::open(path).map_err(|err| format!("Could not read file {:?}: {}", path, err))?;
    let lines = io::BufReader::new(file).lines();
    let lines = LogLines::new(lines);
    let day_collector = DayCollector::new(lines);

    match scope {
        SummaryScope::All => {
            for day in day_collector {
                let day = day?;
                print_day_summary(&mut w, &day.tasks).map_err(map_io_err)?;
                writeln!(&mut w).map_err(map_io_err)?;
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
                print_day_summary(&mut w, &tasks).map_err(map_io_err)?;
                writeln!(&mut w).map_err(map_io_err)?;
            }
        }
    };

    Ok(())
}

fn print_day_summary(mut w: impl io::Write, registry: &TaskRegistry) -> io::Result<()> {
    writeln!(&mut w, "=== {:?}", registry.get_start_time().unwrap())?;
    for task in registry.get_tasks() {
        writeln!(&mut w, "{}", task)?;
    }

    writeln!(
        &mut w,
        "-- Work time: {}",
        format_duration(&registry.get_work_duration())
    )?;

    writeln!(&mut w, "-- Work hours:")?;
    writeln!(&mut w, "on   \toff  \ttime \tpause")?;
    let mut last_off: Option<DateTime<FixedOffset>> = None;
    for (on, off) in registry.get_work_times() {
        let delta = format_duration(&off.sub(*on).to_std().unwrap());
        let pause = match last_off {
            Some(last_off) => format_duration(&on.sub(last_off).to_std().unwrap()),
            None => "".to_string(),
        };
        last_off = Some(*off);
        writeln!(
            &mut w,
            "{}\t{}\t{}\t{}",
            on.format("%H:%M"),
            off.format("%H:%M"),
            delta,
            pause
        )?;
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

fn map_io_err(err: io::Error) -> String {
    err.to_string()
}
