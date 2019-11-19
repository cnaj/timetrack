use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use crate::timelog::LogEvent;
use crate::timelog::TimelogEntry;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum LogLine {
    Entry(TimelogEntry),
    Ignored,
}

impl LogLine {
    fn from_str(line: &str) -> Result<LogLine, String> {
        if line.is_empty() || line.starts_with('#') {
            return Ok(LogLine::Ignored);
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

pub struct DayCollector {
    log_lines: LogLines,
    done: bool,
    buffer: Vec<LogLine>,
}

impl DayCollector {
    pub fn new(log_lines: LogLines) -> DayCollector {
        DayCollector {
            log_lines,
            done: false,
            buffer: Vec::new(),
        }
    }
}

impl Iterator for DayCollector {
    type Item = Result<Vec<LogLine>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        loop {
            match self.log_lines.next() {
                None => {
                    self.done = true;
                    let result: Vec<LogLine> = self.buffer.iter().cloned().collect();
                    self.buffer.clear();
                    return Some(Ok(result));
                }
                Some(line) => match line {
                    Err(err) => {
                        self.done = true;
                        return Some(Err(format!("Input error: {}", err)));
                    }
                    Ok(line) => {
                        match &line {
                            LogLine::Entry(entry) => match entry.event {
                                LogEvent::On => {
                                    let result: Vec<LogLine> =
                                        self.buffer.iter().cloned().collect();
                                    self.buffer.clear();
                                    self.buffer.push(line.clone());
                                    return Some(Ok(result));
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                        self.buffer.push(line.clone());
                    }
                },
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
