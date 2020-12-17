extern crate timetrack;

use std::fs;
use std::path::PathBuf;

use timetrack::cmd;
use timetrack::cmd::SummaryScope;

#[test]
fn test_summaries() {
    let d: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "resources",
        "timetrack.csv",
    ]
    .iter()
    .collect();
    let file = d.to_str().unwrap();

    let expected_path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "resources",
        "expected",
        "day_4_summary.txt",
    ]
    .iter()
    .collect();
    let expected = fs::read_to_string(expected_path).unwrap();

    let mut w: Vec<u8> = Vec::new();
    cmd::summaries(&mut w, file, SummaryScope::Last(1)).unwrap();

    let result = String::from_utf8(w).unwrap();

    assert_eq!(result, expected);
}

#[test]
fn test_worklog() {
    let d: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "resources",
        "timetrack.csv",
    ]
    .iter()
    .collect();
    let file = d.to_str().unwrap();

    let expected_path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "resources",
        "expected",
        "day_3-4_worklog.txt",
    ]
    .iter()
    .collect();
    let expected = fs::read_to_string(expected_path).unwrap();

    let mut w: Vec<u8> = Vec::new();
    cmd::worklog(&mut w, file, SummaryScope::Last(2)).unwrap();

    let result = String::from_utf8(w).unwrap();

    assert_eq!(result, expected);
}
