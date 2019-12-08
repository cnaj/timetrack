use std::fs::File;
use std::io;
use std::io::BufRead;
use std::iter::Enumerate;
use std::path::Path;

use chrono::{DateTime, FixedOffset};

use crate::timelog::LogEvent;
use crate::timelog::TimelogEntry;
use std::fmt::Display;

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
    pub start: Option<DateTime<FixedOffset>>,
    pub lines: Vec<(usize, LogLine)>,
}

pub struct DayCollector<T>
where
    T: Iterator<Item = io::Result<String>>,
{
    log_lines: Enumerate<LogLines<T>>,
    done: bool,
    buffer: Vec<(usize, LogLine)>,
    lookahead: usize,
    start: Option<DateTime<FixedOffset>>,
}

impl<T> DayCollector<T>
where
    T: Iterator<Item = io::Result<String>>,
{
    pub fn new(log_lines: LogLines<T>) -> DayCollector<T> {
        DayCollector {
            log_lines: log_lines.enumerate(),
            done: false,
            buffer: Vec::new(),
            lookahead: 0,
            start: None,
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
            match self.log_lines.next() {
                None => {
                    self.done = true;
                    let lines: Vec<(usize, LogLine)> = self.buffer.drain(..).collect();
                    if !lines.is_empty() {
                        let result = DayCollection {
                            start: self.start.clone(),
                            lines,
                        };
                        return Some(Ok(result));
                    } else {
                        return None;
                    }
                }
                Some(line) => match line {
                    (_, Err(err)) => {
                        self.done = true;
                        return Some(Err(format!("Input error: {}", err)));
                    }
                    (n, Ok(log_line)) => {
                        self.buffer.push((n + 1, log_line.clone()));
                        match log_line {
                            LogLine::Entry(entry) => match entry.event {
                                LogEvent::On => {
                                    if self.start.is_none() {
                                        self.start = Some(entry.time.clone());
                                        self.lookahead = 0;
                                    } else {
                                        let start = self.start.unwrap();
                                        let len = self.buffer.len() - self.lookahead - 1;
                                        self.start = Some(entry.time.clone());
                                        self.lookahead = 0;
                                        let lines: Vec<(usize, LogLine)> =
                                            self.buffer.drain(..len).collect();
                                        let result = DayCollection {
                                            start: Some(start),
                                            lines,
                                        };
                                        return Some(Ok(result));
                                    }
                                }
                                _ => {}
                            },
                            LogLine::Ignored(line) => {
                                if self.start.is_none() || self.lookahead > 0 || !line.is_empty() {
                                    self.lookahead += 1;
                                }
                            }
                        }
                    }
                },
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
