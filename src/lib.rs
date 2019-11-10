extern crate chrono;

use crate::LogAction::{Start, Stop};
use crate::LogEvent::{Pause, Task, Work};
use chrono::{DateTime, FixedOffset};

#[derive(Eq, PartialEq, Debug)]
enum LogEvent {
    Work,
    Pause,
    Task { name: String },
}

#[derive(Eq, PartialEq, Debug)]
enum LogAction {
    Start,
    Stop,
}

#[derive(Eq, PartialEq, Debug)]
struct TimelogEntry {
    time: DateTime<FixedOffset>,
    action: LogAction,
    event: LogEvent,
}

impl TimelogEntry {
    fn new(time: &DateTime<FixedOffset>, action: LogAction, event: LogEvent) -> TimelogEntry {
        TimelogEntry {
            time: *time,
            action,
            event,
        }
    }

    fn of_str(time: &str, action: LogAction, event: LogEvent) -> TimelogEntry {
        TimelogEntry {
            time: DateTime::parse_from_rfc3339(time).unwrap(),
            action,
            event,
        }
    }

    fn parse_from_str<'a>(line: &'a str) -> Result<TimelogEntry, String> {
        let mut part_it: std::str::Split<'a, char> = line.split('\t');

        let time: DateTime<FixedOffset>;
        match part_it.next() {
            Some(time_part) => match DateTime::parse_from_str(time_part, "%Y-%m-%dT%H:%M%z") {
                Ok(parsed) => time = parsed,
                Err(_) => return Err("could not parse time".to_owned()),
            },
            None => return Err("expected time part".to_owned()),
        }

        let action_part = part_it.next().ok_or("expected action part")?;
        let action: LogAction = match action_part {
            "start" => Start,
            "stop" => Stop,
            &_ => return Err("unexpected action: ".to_owned() + action_part),
        };

        let event_part = part_it.next().ok_or("expected event part")?;
        let event: LogEvent = match event_part {
            "work" => Work,
            "pause" => Pause,
            "task" => {
                let name = part_it.next().ok_or("expected task name")?;
                Task {
                    name: name.to_string(),
                }
            }
            &_ => return Err("unexpected event: ".to_owned() + event_part),
        };

        let rest: String = part_it.fold(String::new(), |mut acc, part| {
            acc.push_str(part);
            acc
        });

        if rest.is_empty() {
            Ok(TimelogEntry {
                time,
                action,
                event,
            })
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
        let entry = TimelogEntry::parse_from_str("2019-11-10T16:04+0100\tstart\twork");
        let expected = TimelogEntry {
            time: DateTime::parse_from_rfc3339("2019-11-10T16:04:00+01:00").unwrap(),
            event: Work,
            action: Start,
        };
        assert_eq!(entry, Ok(expected));
    }

    #[test]
    fn test_parse_line_pause() {
        let entry = TimelogEntry::parse_from_str("2019-11-10T16:04+0100\tstop\tpause");
        let expected = TimelogEntry::new(
            &DateTime::parse_from_rfc3339("2019-11-10T16:04:00+01:00").unwrap(),
            Stop,
            Pause,
        );
        assert_eq!(entry, Ok(expected));
    }

    #[test]
    fn test_parse_line_task() {
        let entry =
            TimelogEntry::parse_from_str("2019-11-10T16:04+0100\tstart\ttask\tRefactor code");
        let expected = TimelogEntry::of_str(
            "2019-11-10T16:04:00+01:00",
            Start,
            Task {
                name: "Refactor code".to_string(),
            },
        );
        assert_eq!(entry, Ok(expected));
    }

    #[test]
    fn test_parse_line_trailing() {
        let entry =
            TimelogEntry::parse_from_str("2019-11-10T16:04+0100\tstart\ttask\tfoobar\tthis \tis trailing");
        let expected = Err("unexpected trailing content: this is trailing".to_owned());
        assert_eq!(entry, expected);
    }
}
