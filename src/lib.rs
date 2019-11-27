extern crate chrono;

pub mod fileread;
pub mod taskregistry;
pub mod timelog;

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::fileread::{DayCollector, LogLine, LogLines};
    use crate::taskregistry::{Task, TaskRegistry};

    const BLANK_LINES: &'static str = r#"

"#;

    const COMMENT_LINE: &'static str = r#"# This is a comment
"#;

    const DAY_1: &'static str = r#"# First line comment
2019-11-21T07:30+0100	on
2019-11-21T07:30+0100	start	BACKEND-errors
2019-11-21T09:45+0100	off
2019-11-21T10:20+0100	start	BACKEND-input-parsing
2019-11-21T10:32+0100	rename	BACKEND-error-handling	BACKEND-errors
2019-11-21T11:40+0100	off
2019-11-21T13:00+0100	start	BACKEND-input-parsing
2019-11-21T17:00+0100	off
"#;

    const DAY_2: &'static str = r#"2019-11-22T07:00+0100	on
2019-11-22T07:02+0100	start	BACKEND-error-handling
2019-11-22T07:27+0100	start	BACKEND-input
2019-11-22T07:30+0100	rename	BACKEND-input-parsing
2019-11-22T09:14+0100	off
2019-11-22T09:20+0100	start	BACKEND-input-parsing
2019-11-22T09:32+0100	off
2019-11-22T09:49+0100	start	BACKEND-input-parsing
2019-11-22T10:40+0100	off
2019-11-22T11:06+0100	start	BACKEND-input-parsing
2019-11-22T11:43+0100	start	Daily
2019-11-22T11:51+0100	start	BACKEND-error-handling
2019-11-22T12:48+0100	off
2019-11-22T13:54+0100	continue
2019-11-22T13:58+0100	start	CHORE - instable tests
2019-11-22T14:06+0100	off
2019-11-22T14:06+0100	start	CHORE - instable tests
2019-11-22T15:24+0100	off
"#;

    #[test]
    fn test_split_days() {
        let mut src = String::new();
        src.push_str(DAY_1);
        src.push_str(DAY_2);

        let lines = src.lines()
            .map(|line| Ok(line.to_owned()));
        let lines = LogLines::new(lines);

        let day_collector = DayCollector::new(lines);

        let days: Vec<_> = day_collector.collect();
        assert_eq!(days.len(), 2);

        let day1 = days[0].as_ref().unwrap();
        assert_eq!(day1.start, Some(DateTime::parse_from_rfc3339("2019-11-21T07:30:00+01:00").unwrap()));
        assert_eq!(day1.lines.len(), 9);

        let day2 = days[1].as_ref().unwrap();
        assert_eq!(day2.start, Some(DateTime::parse_from_rfc3339("2019-11-22T07:00:00+01:00").unwrap()));
        assert_eq!(day2.lines.len(), 18);
    }

    #[test]
    fn test_split_days_with_blank() {
        let mut src = String::new();
        src.push_str(BLANK_LINES);
        src.push_str(DAY_1);
        src.push_str(BLANK_LINES);
        src.push_str(COMMENT_LINE);
        src.push_str(DAY_2);
        src.push_str(BLANK_LINES);

        let lines = src.lines()
            .map(|line| Ok(line.to_owned()));
        let lines = LogLines::new(lines);

        let day_collector = DayCollector::new(lines);

        let days: Vec<_> = day_collector.collect();
        assert_eq!(days.len(), 2);

        let day1 = days[0].as_ref().unwrap();
        assert_eq!(day1.start, Some(DateTime::parse_from_rfc3339("2019-11-21T07:30:00+01:00").unwrap()));
        assert_eq!(day1.lines.len(), 13);

        let day2 = days[1].as_ref().unwrap();
        assert_eq!(day2.start, Some(DateTime::parse_from_rfc3339("2019-11-22T07:00:00+01:00").unwrap()));
        assert_eq!(day2.lines.len(), 21);
    }

    #[test]
    fn test_day_1_tasks() {
        let lines = DAY_1.lines()
            .map(|line| Ok(line.to_owned()));
        let lines = LogLines::new(lines);

        let day_collector = DayCollector::new(lines);

        let days: Vec<_> = day_collector.collect();
        assert_eq!(days.len(), 1);

        let day = days[0].as_ref().unwrap();
        assert_eq!(day.start, Some(DateTime::parse_from_rfc3339("2019-11-21T07:30:00+01:00").unwrap()));

        let it = day.lines.iter()
            .filter_map(|line| match &line.1 {
                LogLine::Entry(entry) => Some((line.0, entry.clone())),
                LogLine::Ignored(_) => None,
            });

        let registry = TaskRegistry::build(it).unwrap();

        let expected = [
            Task::new("Pause", 115),
            Task::new("n/n", 0),
            Task::new("BACKEND-error-handling", 135),
            Task::new("BACKEND-input-parsing", 320),
        ];
        assert_eq!(registry.get_tasks(), expected.as_ref());
    }

    #[test]
    fn test_day_2_tasks() {
        let lines = DAY_2.lines()
            .map(|line| Ok(line.to_owned()));
        let lines = LogLines::new(lines);

        let day_collector = DayCollector::new(lines);

        let days: Vec<_> = day_collector.collect();
        assert_eq!(days.len(), 1);

        let day = days[0].as_ref().unwrap();
        assert_eq!(day.start, Some(DateTime::parse_from_rfc3339("2019-11-22T07:00:00+01:00").unwrap()));

        let it = day.lines.iter()
            .filter_map(|line| match &line.1 {
                LogLine::Entry(entry) => Some((line.0, entry.clone())),
                LogLine::Ignored(_) => None,
            });

        let registry = TaskRegistry::build(it).unwrap();

        let expected = [
            Task::new("Pause", 115),
            Task::new("n/n", 6),
            Task::new("BACKEND-error-handling", 82),
            Task::new("BACKEND-input-parsing", 207),
            Task::new("Daily", 8),
            Task::new("CHORE - instable tests", 86),
        ];
        assert_eq!(registry.get_tasks(), expected.as_ref());
    }
}