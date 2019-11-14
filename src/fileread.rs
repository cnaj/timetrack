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
        self.lines.next()
            .map(|line| line
                .map_err(|err| format!("Could not read line: {}", err))
                .and_then(|line| LogLine::from_str(line.as_str()))
            )
    }
}

pub fn read_log_lines<P>(filename: P) -> io::Result<LogLines>
    where P: AsRef<Path>
{
    let file = File::open(filename)?;
    Ok(LogLines { lines: io::BufReader::new(file).lines() })
}
