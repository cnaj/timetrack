use crate::timelog::LogEvent::{Cancel, Resume, Off, On, Rename, Start, Stop};
use chrono::{DateTime, FixedOffset};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum LogEvent {
    On,
    Off,
    Resume,
    Cancel,
    Start(String),
    Stop,
    Rename { to: String, from: Option<String> },
    OffSnapshot,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TimelogEntry {
    pub time: DateTime<FixedOffset>,
    pub event: LogEvent,
}

impl TimelogEntry {
    pub fn new(time: &DateTime<FixedOffset>, event: LogEvent) -> TimelogEntry {
        TimelogEntry { time: *time, event }
    }

    #[allow(dead_code)]
    fn of_str(time: &str, event: LogEvent) -> TimelogEntry {
        TimelogEntry {
            time: DateTime::parse_from_rfc3339(time).unwrap(),
            event,
        }
    }

    pub fn parse_from_str<'a>(line: &'a str) -> Result<TimelogEntry, String> {
        let mut part_it: std::str::Split<'a, char> = line.split('\t');

        let time: DateTime<FixedOffset>;
        match part_it.next() {
            Some(time_part) => match DateTime::parse_from_str(time_part, "%Y-%m-%dT%H:%M%z") {
                Ok(parsed) => time = parsed,
                Err(_) => return Err("could not parse time".to_owned()),
            },
            None => return Err("expected time part".to_owned()),
        }

        let event_part = part_it.next().ok_or("expected event part")?;
        let event: LogEvent = match event_part {
            "on" => On,
            "off" => Off,
            "resume" => Resume,
            "cancel" => Cancel,
            "start" => {
                let name = part_it.next().ok_or("expected task name")?;
                Start(name.to_owned())
            }
            "stop" => Stop,
            "rename" => {
                let to = part_it
                    .next()
                    .ok_or("expected target task name")?
                    .to_owned();
                let from = part_it.next().map(|s| s.to_owned());
                Rename { to, from }
            }
            &_ => return Err("unexpected event: ".to_owned() + event_part),
        };

        let rest: String = part_it.fold(String::new(), |mut acc, part| {
            acc.push_str(part);
            acc
        });

        if rest.is_empty() {
            Ok(TimelogEntry { time, event })
        } else {
            Err("unexpected trailing content: ".to_owned() + &rest)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn test_parse_line_work() {
        let entry = TimelogEntry::parse_from_str("2019-11-10T16:04+0100\ton");
        let expected = TimelogEntry {
            time: DateTime::parse_from_rfc3339("2019-11-10T16:04:00+01:00").unwrap(),
            event: On,
        };
        assert_eq!(entry, Ok(expected));
    }

    #[test]
    fn test_parse_line_resume() {
        let entry = TimelogEntry::parse_from_str("2019-11-10T16:04+0100\tresume");
        let expected = TimelogEntry::new(
            &DateTime::parse_from_rfc3339("2019-11-10T16:04:00+01:00").unwrap(),
            Resume,
        );
        assert_eq!(entry, Ok(expected));
    }

    #[test]
    fn test_parse_line_task() {
        let entry = TimelogEntry::parse_from_str("2019-11-10T16:04+0100\tstart\tRefactor code");
        let expected = TimelogEntry::of_str(
            "2019-11-10T16:04:00+01:00",
            Start("Refactor code".to_owned()),
        );
        assert_eq!(entry, Ok(expected));
    }

    #[test]
    fn test_parse_line_trailing() {
        let entry = TimelogEntry::parse_from_str(
            "2019-11-10T16:04+0100\tstart\tfoobar\tthis \tis trailing",
        );
        let expected = Err("unexpected trailing content: this is trailing".to_owned());
        assert_eq!(entry, expected);
    }
}
