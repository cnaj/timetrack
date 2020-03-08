use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use crate::taskregistry::{TaskRegistry, TaskRegistryBuilder};
use crate::timelog::TimelogEntry;
use std::fmt::Display;
use std::iter::Enumerate;

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
    lines: Enumerate<T>,
}

impl<T, E> LogLines<T>
where
    T: Iterator<Item = Result<String, E>>,
    E: Display,
{
    pub fn new(src: T) -> LogLines<T> {
        LogLines {
            lines: src.enumerate(),
        }
    }
}

impl<T, E> Iterator for LogLines<T>
where
    T: Iterator<Item = Result<String, E>>,
    E: Display,
{
    type Item = (usize, Result<LogLine, String>);

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|(n, line)| {
            let line_nr = n + 1;
            let line = line
                .map_err(|err| format!("Could not read line nr. {}: {}", line_nr, err))
                .and_then(|line| LogLine::from_str(line.as_str()));
            (line_nr, line)
        })
    }
}

#[derive(Debug, Clone)]
pub struct DayCollection {
    pub tasks: TaskRegistry,
    pub lines: Vec<(usize, LogLine)>,
}

pub struct DayCollector<I> {
    it: I,
    builder: TaskRegistryBuilder,
    done: bool,
    buffer: Vec<(usize, LogLine)>,
    lookahead: usize,
}

impl<I, E> DayCollector<I>
where
    I: Iterator<Item = (usize, Result<LogLine, E>)>,
    E: Display,
{
    pub fn new(it: I) -> DayCollector<I> {
        DayCollector {
            it,
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

    fn process_next_line(
        &mut self,
        n: usize,
        log_line: LogLine,
    ) -> Option<Result<DayCollection, String>> {
        self.buffer.push((n + 1, log_line.clone()));

        match log_line {
            LogLine::Entry(entry) => {
                let result = match self.builder.add_entry(&entry) {
                    Err(err) => Some(Err(format!(
                        "{} (while processing {:?} in line {})",
                        err, entry, n
                    ))),
                    Ok(tasks_opt) => tasks_opt
                        .map(|tasks| {
                            let len = self.buffer.len() - self.lookahead - 1;
                            let lines: Vec<(usize, LogLine)> = self.buffer.drain(..len).collect();
                            let result = DayCollection {
                                tasks: tasks.clone(),
                                lines,
                            };

                            result
                        })
                        .map(|res| Ok(res)),
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

impl<I, E> Iterator for DayCollector<I>
where
    I: Iterator<Item = (usize, Result<LogLine, E>)>,
    E: Display,
{
    type Item = Result<DayCollection, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        loop {
            let next = self.it.next();

            let (n, line) = match next {
                None => return self.process_eof(),
                Some((n, line_res)) => match line_res {
                    Err(err) => return Some(Err(format!("Input error on line {}: {}", n, err))),
                    Ok(line) => (n, line),
                },
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
