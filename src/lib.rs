extern crate chrono;

pub mod fileread;
pub mod taskregistry;
pub mod timelog;

#[cfg(test)]
mod tests {
    use crate::fileread::{LogLines, DayCollector, LogLine};
    use crate::taskregistry::{TaskRegistry, Task};
    use chrono::DateTime;

    const DAY_0: &'static str = r#"# First line comment
2019-11-22T07:00+0100	on
2019-11-22T07:02+0100	start	BACKEND-error-handling
2019-11-22T07:27+0100	start	BACKEND-input-parsing
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
    fn test_day_tasks() {
        let lines = DAY_0.lines()
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