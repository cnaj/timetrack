use crate::taskregistry::TaskRegistry;
use chrono::{DateTime, FixedOffset};
use std::io;
use std::ops::Sub;
use std::time::Duration;

pub fn tasks(mut w: impl io::Write, registry: &TaskRegistry) -> io::Result<()> {
    let tasks = registry.get_tasks();

    writeln!(&mut w, "#\ttime\ttask name")?;
    for (n, task) in tasks.iter().enumerate().skip(1) {
        writeln!(&mut w, "{}\t{}", n, task)?;
    }
    writeln!(
        &mut w,
        "\t{}\ttotal work time",
        format_duration(&registry.get_work_duration())
    )?;
    Ok(())
}

pub fn day_summary(mut w: impl io::Write, registry: &TaskRegistry) -> io::Result<()> {
    writeln!(&mut w, "=== {:?}", registry.get_start_time().unwrap())?;
    for task in registry.get_tasks() {
        writeln!(&mut w, "{}", task)?;
    }

    writeln!(
        &mut w,
        "-- Work time: {}",
        format_duration(&registry.get_work_duration())
    )?;

    writeln!(&mut w, "-- Work hours:")?;
    writeln!(&mut w, "on   \toff  \ttime \tpause")?;
    let mut last_off: Option<DateTime<FixedOffset>> = None;
    for (on, off) in registry.get_work_times() {
        let delta = format_duration(&off.sub(*on).to_std().unwrap());
        let pause = match last_off {
            Some(last_off) => format_duration(&on.sub(last_off).to_std().unwrap()),
            None => "".to_string(),
        };
        last_off = Some(*off);
        writeln!(
            &mut w,
            "{}\t{}\t{}\t{}",
            on.format("%H:%M"),
            off.format("%H:%M"),
            delta,
            pause
        )?;
    }

    Ok(())
}

fn format_duration(work_time: &Duration) -> String {
    let secs = work_time.as_secs();
    let mins = secs / 60;
    let m = mins % 60;
    let h = mins / 60;
    format!("{:02}:{:02}", h, m)
}
