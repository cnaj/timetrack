use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use crate::timelog::LogEvent;
use crate::timelog::TimelogEntry;
use chrono::{DateTime, FixedOffset};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum LogLine {
    Entry(TimelogEntry),
    Ignored(String),
}

impl LogLine {
    fn from_str(line: &str) -> Result<LogLine, String> {
        if line.is_empty() || line.starts_with('#') {
            return Ok(LogLine::Ignored(line.to_owned()));
        }

        let entry = TimelogEntry::parse_from_str(line)?;
        Ok(LogLine::Entry(entry))
    }
}

pub struct LogLines {
    lines: io::Lines<io::BufReader<File>>,
}

impl Iterator for LogLines {
    type Item = Result<LogLine, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|line| {
            line.map_err(|err| format!("Could not read line: {}", err))
                .and_then(|line| LogLine::from_str(line.as_str()))
        })
    }
}

#[derive(Debug, Clone)]
pub struct DayCollection {
    pub start: Option<DateTime<FixedOffset>>,
    pub lines: Vec<(usize, LogLine)>,
}

pub struct DayCollector {
    log_lines: LogLines,
    line_count: usize,
    done: bool,
    buffer: Vec<(usize, LogLine)>,
    lookahead: usize,
    start: Option<DateTime<FixedOffset>>,
}

impl DayCollector {
    pub fn new(log_lines: LogLines) -> DayCollector {
        DayCollector {
            log_lines,
            line_count: 0,
            done: false,
            buffer: Vec::new(),
            lookahead: 0,
            start: None,
        }
    }
}

impl Iterator for DayCollector {
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
                Some(line) => {
                    self.line_count += 1;
                    match line {
                        Err(err) => {
                            self.done = true;
                            return Some(Err(format!("Input error: {}", err)));
                        }
                        Ok(line) => {
                            self.buffer.push((self.line_count, line.clone()));
                            match &line {
                                LogLine::Entry(entry) => match entry.event {
                                    LogEvent::On => {
                                        if self.start.is_none() {
                                            self.start = Some(entry.time.clone());
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
                                    if self.start.is_none()
                                        || self.lookahead > 0
                                        || !line.is_empty()
                                    {
                                        self.lookahead += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn read_log_lines<P>(filename: P) -> io::Result<LogLines>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(LogLines {
        lines: io::BufReader::new(file).lines(),
    })
}
