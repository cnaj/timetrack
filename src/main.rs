use std::io;

use clap::{App, Arg, ArgMatches, SubCommand};

use timetrack::cmd;
use timetrack::cmd::SummaryScope;

fn main() -> Result<(), String> {
    let matches = App::new("timetrack")
        .about("Command-line time tracking tool")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Path to input file")
                .required(true)
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("last-active").about("Displays the last recorded active task"),
        )
        .subcommand(
            SubCommand::with_name("summary")
                .about("Displays a task and time summary per work day.")
                .subcommand(
                    SubCommand::with_name("all").about("Displays tasks for all available days"),
                )
                .subcommand(
                    SubCommand::with_name("last")
                        .about("Displays tasks of the last days")
                        .arg(Arg::with_name("number").default_value("1")),
                ),
        )
        .subcommand(SubCommand::with_name("tasks").about("Displays a list of recorded tasks"))
        .get_matches();

    let file_path = matches.value_of("file").unwrap();

    let mut w = io::stdout();
    match matches.subcommand() {
        ("last-active", Some(_)) => cmd::last_active(&mut w, file_path)?,
        ("summary", Some(sub_matches)) => cmd_summary(&mut w, sub_matches, file_path)?,
        ("tasks", Some(_)) => cmd::tasks(&mut w, file_path)?,
        _ => cmd::summaries(&mut w, file_path, SummaryScope::Last(1))?,
    };

    Ok(())
}

fn cmd_summary(mut w: impl io::Write, matches: &ArgMatches, file_path: &str) -> Result<(), String> {
    let scope = match matches.subcommand() {
        ("all", Some(_)) => SummaryScope::All,
        ("last", Some(last_matches)) => match last_matches.value_of("number") {
            None => SummaryScope::Last(1),
            Some(number) => match number.parse::<usize>() {
                Ok(n) => SummaryScope::Last(n),
                Err(e) => return Err(format!("Invalid number given: {}", e)),
            },
        },
        _ => SummaryScope::Last(1),
    };

    cmd::summaries(&mut w, file_path, scope)
}
