use std::io;
use std::path::Path;
use std::fs::File;
use std::io::BufRead;

use crate::timelog::TimelogEntry;

#[derive(Eq, PartialEq, Debug)]
pub enum LogLine {
    Entry(TimelogEntry),
    Ignored,
}

pub struct LogLines {
    lines: io::Lines<io::BufReader<File>>,
}

impl Iterator for LogLines {
    type Item = Result<LogLine, String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            None => None,
            Some(line) => {
                match line {
                    Err(err) => return Some(Err(format!("Could not read line: {}", err))),
                    Ok(line) => {
                        if line.is_empty() || line.starts_with('#') {
                            return Some(Ok(LogLine::Ignored));
                        }

                        match TimelogEntry::parse_from_str(line.as_ref()) {
                            Err(err) => Some(Err(format!("Unknown log entry: {}", err))),
                            Ok(log_entry) => Some(Ok(LogLine::Entry(log_entry))),
                        }
                    }
                }
            }
        }
    }
}

pub fn read_log_lines<P>(filename: P) -> io::Result<LogLines>
    where P: AsRef<Path>
{
    let file = File::open(filename)?;
    Ok(LogLines { lines: io::BufReader::new(file).lines() })
}
