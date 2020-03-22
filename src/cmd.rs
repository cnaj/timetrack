use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::BufRead;

use crate::fileread::{DayCollector, LogLines};
use crate::print;

pub enum SummaryScope {
    All,
    Last(usize),
}

pub fn last_active(mut w: impl io::Write, path: &str) -> Result<(), String> {
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

pub fn tasks(mut w: impl io::Write, path: &str) -> Result<(), String> {
    let file =
        File::open(path).map_err(|err| format!("Could not read file {:?}: {}", path, err))?;
    let lines = io::BufReader::new(file).lines();
    let lines = LogLines::new(lines);
    let day_collector = DayCollector::new(lines);

    match day_collector.last() {
        Some(day_result) => {
            let registry = day_result?.tasks;
            print::tasks(&mut w, &registry).map_err(map_io_err)?;
        }
        None => {}
    };

    Ok(())
}

pub fn summaries(mut w: impl io::Write, path: &str, scope: SummaryScope) -> Result<(), String> {
    let file =
        File::open(path).map_err(|err| format!("Could not read file {:?}: {}", path, err))?;
    let lines = io::BufReader::new(file).lines();
    let lines = LogLines::new(lines);
    let day_collector = DayCollector::new(lines);

    match scope {
        SummaryScope::All => {
            for day in day_collector {
                let day = day?;
                print::day_summary(&mut w, &day.tasks).map_err(map_io_err)?;
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
                print::day_summary(&mut w, &tasks).map_err(map_io_err)?;
                writeln!(&mut w).map_err(map_io_err)?;
            }
        }
    };

    Ok(())
}

fn map_io_err(err: io::Error) -> String {
    err.to_string()
}
