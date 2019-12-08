use std::fs::File;
use std::io;
use std::io::BufRead;
use std::iter::Enumerate;
use std::path::Path;

use crate::timelog::TimelogEntry;
use std::fmt::Display;
use crate::taskregistry::{TaskRegistryBuilder, TaskRegistry};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum LogLine {
    Entry(TimelogEntry),
    Ignored(String),
}

impl LogLine {
    pub fn from_str(line: &str) -> Result<LogLine, String> {
        if line.is_empty() || line.starts_with('#') {
            return Ok(LogLine::Ignored(line.to_owned()));
        }

        let entry = TimelogEntry::parse_from_str(line)?;
        Ok(LogLine::Entry(entry))
    }
}

pub struct LogLines<T> {
    lines: T,
}

impl<T> LogLines<T> {
    pub fn new(src: T) -> LogLines<T> {
        LogLines { lines: src }
    }
}

impl<T> Iterator for LogLines<T>
where
    T: Iterator<Item = io::Result<String>>,
{
    type Item = Result<LogLine, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|line| {
            line.map_err(|err| format!("Could not read line: {}", err))
                .and_then(|line| LogLine::from_str(line.as_str()))
                .map(|line| line)
        })
    }
}

pub struct LogEntries<T> {
    lines: T,
    line_count: usize,
}

impl<T, E> LogEntries<T>
where
    T: Iterator<Item = Result<String, E>>,
    E: Display,
{
    pub fn new(src: T) -> LogEntries<T> {
        LogEntries {
            lines: src,
            line_count: 0,
        }
    }
}

impl<T, E> Iterator for LogEntries<T>
where
    T: Iterator<Item = Result<String, E>>,
    E: Display,
{
    type Item = (usize, Result<TimelogEntry, String>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.lines.next() {
                None => return None,
                Some(result) => {
                    self.line_count += 1;
                    match result {
                        Err(err) => return Some((self.line_count, Err(err.to_string()))),
                        Ok(line) => match LogLine::from_str(line.as_str()) {
                            Err(err) => return Some((self.line_count, Err(err))),
                            Ok(LogLine::Entry(entry)) => {
                                return Some((self.line_count, Ok(entry)));
                            }
                            Ok(LogLine::Ignored(_)) => {}
                        },
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DayCollection {
    pub tasks: Option<TaskRegistry>,
    pub lines: Vec<(usize, LogLine)>,
}

pub struct DayCollector<T>
where
    T: Iterator<Item = io::Result<String>>,
{
    log_lines: Enumerate<LogLines<T>>,
    builder: TaskRegistryBuilder,
    done: bool,
    buffer: Vec<(usize, LogLine)>,
    lookahead: usize,
}

impl<T> DayCollector<T>
where
    T: Iterator<Item = io::Result<String>>,
{
    pub fn new(log_lines: LogLines<T>) -> DayCollector<T> {
        DayCollector {
            log_lines: log_lines.enumerate(),
            builder: TaskRegistryBuilder::new(),
            done: false,
            buffer: Vec::new(),
            lookahead: 0,
        }
    }

    fn process_eof(&mut self) -> Option<Result<DayCollection, String>> {
        self.done = true;
        let lines: Vec<(usize, LogLine)> = self.buffer.drain(..).collect();
        if !lines.is_empty() {
            let result = DayCollection {
                tasks: self.builder.finish(),
                lines,
            };
            Some(Ok(result))
        } else {
            None
        }
    }

    fn process_next_line(&mut self, n: usize, log_line: LogLine)
                         -> Option<Result<DayCollection, String>>
    {
        self.buffer.push((n + 1, log_line.clone()));

        match log_line {
            LogLine::Entry(entry) => {
                let result = match self.builder.add_entry(&entry) {
                    Err(err) => Some(Err(format!("{} (while processing {:?} in line {})", err, entry, n))),
                    Ok(tasks_opt) => tasks_opt
                        .map(|tasks| {
                            let len = self.buffer.len() - self.lookahead - 1;
                            let lines: Vec<(usize, LogLine)> =
                                self.buffer.drain(..len).collect();
                            let result = DayCollection {
                                tasks: Some(tasks.clone()),
                                lines,
                            };

                            result
                        })
                        .map(|res| Ok(res))
                };

                self.lookahead = 0;
                result
            }
            LogLine::Ignored(line) => {
                if self.lookahead > 0 || !line.is_empty() {
                    self.lookahead += 1;
                }
                None
            }
        }
    }
}

impl<T> Iterator for DayCollector<T>
where
    T: Iterator<Item = io::Result<String>>,
{
    type Item = Result<DayCollection, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        loop {
            let next = self.log_lines.next();

            let (n, line) = match next {
                None => return self.process_eof(),
                Some((n, line_res)) => match line_res {
                    Err(err) => return Some(Err(format!("Input error on line {}: {}", n, err))),
                    Ok(line) => (n, line)
                }
            };

            let result = self.process_next_line(n, line);
            if result.is_some() {
                return result;
            }
        }
    }
}

pub fn read_log_lines<P>(filename: P) -> io::Result<LogLines<io::Lines<io::BufReader<File>>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(LogLines::new(io::BufReader::new(file).lines()))
}
