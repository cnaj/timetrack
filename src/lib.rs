extern crate chrono;

pub mod cmd;
pub mod fileread;
pub mod print;
pub mod taskregistry;
pub mod timelog;

#[cfg(test)]
mod tests {
    use std::io;
    use std::time::Duration;

    use chrono::DateTime;

    use crate::fileread::{DayCollection, DayCollector, LogLines};
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

# Pause

2019-11-22T09:49+0100	start	BACKEND-input-parsing
2019-11-22T10:40+0100	off
2019-11-22T11:06+0100	start	BACKEND-input-parsing
2019-11-22T11:43+0100	start	Daily
2019-11-22T11:51+0100	start	BACKEND-error-handling
2019-11-22T12:48+0100	off
2019-11-22T13:54+0100	resume
2019-11-22T13:58+0100	start	CHORE - instable tests
2019-11-22T14:06+0100	off
2019-11-22T14:06+0100	start	CHORE - instable tests
2019-11-22T15:24+0100	off
"#;

    const DAY_3: &'static str = r#"2019-11-26T07:00+0100	on
2019-11-26T07:10+0100	start	FRONTEND - error handling
2019-11-26T07:34+0100	start	BACKEND - query endpoint
2019-11-26T07:48+0100	off
2019-11-26T08:12+0100	resume
2019-11-26T08:14+0100	start	time logging
2019-11-26T08:20+0100	start	CHORE - build system
2019-11-26T09:19+0100	start	Team discussion
2019-11-26T09:30+0100	start	BACKEND - query endpoint
2019-11-26T09:51+0100	off
2019-11-26T10:43+0100	resume
2019-11-26T10:58+0100	start	time logging
2019-11-26T11:10+0100	start	BACKEND - query endpoint
2019-11-26T11:23+0100	start	BACKEND - integration tests
2019-11-26T11:30+0100	start	Daily
2019-11-26T11:45+0100	off
2019-11-26T12:28+0100	start	BACKEND - integration tests
2019-11-26T13:26+0100	start	backlog
2019-11-26T13:32+0100	start	CHORE - build system
2019-11-26T13:56+0100	stop
2019-11-26T14:01+0100	start	backlog
2019-11-26T15:57+0100	stop
2019-11-26T16:13+0100	start	UI JWT timeout
2019-11-26T16:30+0100	start	Bugfix Export
2019-11-26T16:51+0100	start	UI JWT timeout
2019-11-26T17:36+0100	start	BACKEND - query endpoint
2019-11-26T17:53+0100	off
"#;

    const DAY_4: &'static str = r#"2019-11-28T08:55+0100	on
2019-11-28T09:08+0100	start	Bugfix Export
2019-11-28T09:30+0100	start	Sprint planning
2019-11-28T10:15+0100	start	CHORE - Build system
2019-11-28T10:52+0100	start	Bugfix Export
2019-11-28T10:56+0100	stop
2019-11-28T11:07+0100	start	BACKEND - logging framework
2019-11-28T11:34+0100	start	FRONTEND - translations
2019-11-28T11:45+0100	start	Daily
2019-11-28T12:05+0100	off
2019-11-28T12:53+0100	resume
2019-11-28T12:58+0100	start	Sprint Retro
2019-11-28T14:43+0100	start	BACKEND - logging framework
2019-11-28T15:24+0100	start	FRONTEND - translations
2019-11-28T15:31+0100	stop
2019-11-28T15:40+0100	start	FRONTEND - release notes
2019-11-28T16:21+0100	start	BACKEND - logging framework
2019-11-28T17:52+0100	stop
2019-11-28T18:07+0100	off
"#;

    #[test]
    fn test_split_days() {
        let mut src = String::new();
        src.push_str(DAY_1);
        src.push_str(DAY_2);

        let days = into_days(src);
        assert_eq!(days.len(), 2);

        let day1 = &days[0];
        assert_eq!(
            day1.tasks.get_start_time().unwrap(),
            DateTime::parse_from_rfc3339("2019-11-21T07:30:00+01:00").unwrap()
        );
        assert_eq!(day1.lines.len(), 9);

        let day2 = &days[1];
        assert_eq!(
            day2.tasks.get_start_time().unwrap(),
            DateTime::parse_from_rfc3339("2019-11-22T07:00:00+01:00").unwrap()
        );
        assert_eq!(day2.lines.len(), 21);
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

        let days = into_days(src);
        assert_eq!(days.len(), 2);

        let day1 = &days[0];
        assert_eq!(
            day1.tasks.get_start_time().unwrap(),
            DateTime::parse_from_rfc3339("2019-11-21T07:30:00+01:00").unwrap()
        );
        assert_eq!(day1.lines.len(), 13);

        let day2 = &days[1];
        assert_eq!(
            day2.tasks.get_start_time().unwrap(),
            DateTime::parse_from_rfc3339("2019-11-22T07:00:00+01:00").unwrap()
        );
        assert_eq!(day2.lines.len(), 24);
    }

    #[test]
    fn test_day_1_tasks() {
        let days = into_days(DAY_1);
        assert_eq!(days.len(), 1);

        let registry = &days[0].tasks;
        assert_day1_tasks(&registry);
    }

    #[test]
    fn test_day_2_tasks() {
        let days = into_days(DAY_2);
        assert_eq!(days.len(), 1);

        let registry = &days[0].tasks;
        assert_day2_tasks(registry);
    }

    #[test]
    fn test_day_1_and_2_tasks() {
        let mut src = String::new();
        src.push_str(BLANK_LINES);
        src.push_str(DAY_1);
        src.push_str(BLANK_LINES);
        src.push_str(COMMENT_LINE);
        src.push_str(DAY_2);
        src.push_str(BLANK_LINES);

        let days = into_days(src);
        assert_eq!(days.len(), 2);

        assert_day1_tasks(&days[0].tasks);
        assert_day2_tasks(&days[1].tasks);
    }

    #[test]
    fn test_work_time() {
        let days = into_days(DAY_3);
        assert_eq!(days.len(), 1);

        assert_day3_work_times(&days[0].tasks);
    }

    #[test]
    fn test_last_active() {
        let days = into_days(DAY_1);
        let tasks = &days[0].tasks;

        let expected = Task::new("BACKEND-input-parsing", 320);
        assert_eq!(tasks.get_last_active(), Some(expected));
    }

    #[test]
    fn test_last_active_nn() {
        let days = into_days(DAY_4);
        let tasks = &days[0].tasks;

        assert_eq!(tasks.get_last_active(), None);
    }

    fn assert_day1_tasks(registry: &TaskRegistry) {
        let expected = [
            Task::new("Pause", 115),
            Task::new("n/n", 0),
            Task::new("BACKEND-error-handling", 135),
            Task::new("BACKEND-input-parsing", 320),
        ];
        assert_eq!(registry.get_tasks(), expected.as_ref());
    }

    fn assert_day2_tasks(registry: &TaskRegistry) {
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

    fn assert_day3_work_times(registry: &TaskRegistry) {
        let expected = [
            (
                DateTime::parse_from_rfc3339("2019-11-26T07:00:00+01:00").unwrap(),
                DateTime::parse_from_rfc3339("2019-11-26T07:48:00+01:00").unwrap(),
            ),
            (
                DateTime::parse_from_rfc3339("2019-11-26T08:12:00+01:00").unwrap(),
                DateTime::parse_from_rfc3339("2019-11-26T09:51:00+01:00").unwrap(),
            ),
            (
                DateTime::parse_from_rfc3339("2019-11-26T10:43:00+01:00").unwrap(),
                DateTime::parse_from_rfc3339("2019-11-26T11:45:00+01:00").unwrap(),
            ),
            (
                DateTime::parse_from_rfc3339("2019-11-26T12:28:00+01:00").unwrap(),
                DateTime::parse_from_rfc3339("2019-11-26T17:53:00+01:00").unwrap(),
            ),
        ];
        assert_eq!(registry.get_work_times(), expected.as_ref());
        assert_eq!(
            registry.get_work_duration(),
            Duration::from_secs(8 * 3600 + 54 * 60)
        );
    }

    fn into_days<T: AsRef<str>>(src: T) -> Vec<DayCollection> {
        let lines = src
            .as_ref()
            .lines()
            .map(|line| io::Result::Ok(line.to_owned()));
        let lines = LogLines::new(lines);
        let day_collector = DayCollector::new(lines);

        let days_result: Result<Vec<_>, _> = day_collector.collect();
        days_result.unwrap()
    }
}
