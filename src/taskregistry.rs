use std::collections::HashMap;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::mem::replace;
use std::ops::Sub;
use std::time::Duration;

use chrono::{DateTime, FixedOffset, Local, Timelike};

use crate::taskregistry::State::{DayTracking, Idle, TaskActive};
use crate::timelog::{LogEvent, TimelogEntry};

const PAUSE_TASK_NAME: &str = "Pause";
const UNDEFINED_TASK_NAME: &str = "n/n";

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Task {
    pub name: String,
    pub duration: Duration,
    pub active: bool,
}

impl Task {
    pub fn new(name: impl ToString, duration_mins: u64) -> Task {
        Task {
            name: name.to_string(),
            duration: Duration::from_secs(duration_mins * 60),
            active: false,
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let secs = self.duration.as_secs();
        let mins = secs / 60;
        let m = mins % 60;
        let h = mins / 60;
        let a = if self.active { "*" } else { "" };
        write!(f, "{:02}:{:02}{}\t{}", h, m, a, self.name)
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum State {
    Idle,
    DayTracking,
    TaskActive,
}

pub struct TaskRegistryBuilder {
    start_time: Option<DateTime<FixedOffset>>,
    state: State,
    current_task_name: Option<String>,
    work_start_time: Option<DateTime<FixedOffset>>,
    task_registry: TaskRegistry,
}

impl TaskRegistryBuilder {
    pub fn new() -> TaskRegistryBuilder {
        TaskRegistryBuilder {
            start_time: None,
            state: Idle,
            current_task_name: None,
            work_start_time: None,
            task_registry: TaskRegistry::new(),
        }
    }

    pub fn add_entry(&mut self, entry: &TimelogEntry) -> Result<Option<TaskRegistry>, String> {
        let mut result = None;
        self.state = match self.state {
            Idle => match &entry.event {
                LogEvent::On => {
                    if !self.task_registry.work_times.is_empty() {
                        result = Some(replace(&mut self.task_registry, TaskRegistry::new()));
                    }
                    self.start_work_time(entry);
                    self.start_task(&entry.time, PAUSE_TASK_NAME);
                    self.stop_task(PAUSE_TASK_NAME, &entry.time)?;
                    self.start_task(&entry.time, UNDEFINED_TASK_NAME);
                    DayTracking
                }
                LogEvent::Continue => {
                    self.start_work_time(entry);
                    self.stop_task(PAUSE_TASK_NAME, &entry.time)?;
                    self.start_task(&entry.time, UNDEFINED_TASK_NAME);
                    DayTracking
                }
                LogEvent::Start(name) => {
                    self.start_work_time(entry);
                    self.stop_task(PAUSE_TASK_NAME, &entry.time)?;
                    self.start_task(&entry.time, name);
                    TaskActive
                }
                _ => {
                    return Err(format!(
                        "Invalid event {:?} in state {:?}",
                        entry.event, self.state
                    ));
                }
            },
            DayTracking => match &entry.event {
                LogEvent::Off => {
                    self.stop_task(UNDEFINED_TASK_NAME, &entry.time)?;
                    self.start_task(&entry.time, PAUSE_TASK_NAME);
                    self.stop_work_time(entry);
                    Idle
                }
                LogEvent::OffSnapshot => {
                    self.record_task_time(UNDEFINED_TASK_NAME, &entry.time, true)?;
                    self.stop_work_time(entry);
                    Idle
                }
                LogEvent::Start(name) => {
                    self.stop_task(UNDEFINED_TASK_NAME, &entry.time)?;
                    self.start_task(&entry.time, name);
                    TaskActive
                }
                _ => {
                    return Err(format!(
                        "Invalid event {:?} in state {:?}",
                        entry.event, self.state
                    ));
                }
            },
            TaskActive => match &entry.event {
                LogEvent::Stop => {
                    self.stop_current_task(&entry.time)?;
                    self.start_task(&entry.time, UNDEFINED_TASK_NAME);
                    DayTracking
                }
                LogEvent::Off => {
                    self.stop_current_task(&entry.time)?;
                    self.start_task(&entry.time, PAUSE_TASK_NAME);
                    self.stop_work_time(entry);
                    Idle
                }
                LogEvent::OffSnapshot => {
                    let name = self.current_task_name.as_ref().unwrap().to_string();
                    self.record_task_time(&name, &entry.time, true)?;
                    self.stop_work_time(entry);
                    Idle
                }
                LogEvent::Start(name) => {
                    self.stop_current_task(&entry.time)?;
                    self.start_task(&entry.time, name);
                    TaskActive
                }
                LogEvent::Rename { to, from } => {
                    let name = from
                        .as_ref()
                        .or(self.current_task_name.as_ref())
                        .ok_or(format!(
                            "No task active while trying to rename current task"
                        ))?;
                    self.task_registry.rename_task(to, name)?;
                    if from.is_none() {
                        self.current_task_name = Some(to.to_owned());
                    }
                    TaskActive
                }
                _ => {
                    return Err(format!(
                        "Invalid event {:?} in state {:?}",
                        entry.event, self.state
                    ));
                }
            },
        };

        Ok(result)
    }

    pub fn finish(&mut self) -> TaskRegistry {
        if self.state != Idle {
            let now = Local::now()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            self.add_entry(&TimelogEntry::new(&now.into(), LogEvent::OffSnapshot))
                .unwrap();
        }

        self.start_time = None;
        replace(&mut self.task_registry, TaskRegistry::new())
    }

    fn start_work_time(&mut self, entry: &TimelogEntry) {
        self.work_start_time = Some(entry.time);
    }

    fn stop_work_time(&mut self, entry: &TimelogEntry) {
        self.task_registry
            .add_work_time(self.work_start_time.unwrap(), entry.time);
        self.work_start_time = None;
    }

    fn stop_current_task(&mut self, time: &DateTime<FixedOffset>) -> Result<(), String> {
        let name = self.current_task_name.as_ref().unwrap().to_string();
        self.stop_task(&name, time)?;
        Ok(())
    }

    fn stop_task(&mut self, name: &str, time: &DateTime<FixedOffset>) -> Result<(), String> {
        self.record_task_time(name, time, false)
    }

    fn record_task_time(
        &mut self,
        name: &str,
        time: &DateTime<FixedOffset>,
        keep_active: bool,
    ) -> Result<(), String> {
        let time_diff = *time - self.start_time.unwrap();
        let duration: Duration = time_diff
            .to_std()
            .map_err(|e| format!("Non-continuous timestamp: {}", e))?;
        self.task_registry
            .record_task_time(name, duration, keep_active)
    }

    fn start_task<T: ToString + AsRef<str>>(
        &mut self,
        time: &DateTime<FixedOffset>,
        name: T,
    ) -> () {
        self.start_time = Some(*time);
        self.current_task_name = Some(name.to_string());
        let active = PAUSE_TASK_NAME != name.as_ref();
        self.task_registry.add_task(name, active);
    }
}

#[derive(Debug, Clone)]
pub struct TaskRegistry {
    tasks: Vec<Task>,
    names: HashMap<String, usize>,
    work_times: Vec<(DateTime<FixedOffset>, DateTime<FixedOffset>)>,
    work_duration: Duration,
}

impl TaskRegistry {
    fn new() -> TaskRegistry {
        TaskRegistry {
            tasks: Vec::new(),
            names: HashMap::new(),
            work_times: Vec::new(),
            work_duration: Duration::from_secs(0),
        }
    }

    pub fn get_tasks(&self) -> &[Task] {
        self.tasks.as_slice()
    }

    pub fn get_start_time(&self) -> Result<DateTime<FixedOffset>, String> {
        let times = self
            .work_times
            .first()
            .ok_or_else(|| format!("No work times recorded"))?;
        Ok(times.0)
    }

    pub fn get_work_times(&self) -> &[(DateTime<FixedOffset>, DateTime<FixedOffset>)] {
        self.work_times.as_slice()
    }

    pub fn get_work_duration(&self) -> Duration {
        self.work_duration
    }

    fn record_task_time(
        &mut self,
        name: &str,
        active_time: Duration,
        keep_active: bool,
    ) -> Result<(), String> {
        let i = self
            .names
            .get(name)
            .ok_or(format!("Couldn't find task name '{}'", name))?;
        let task = self.tasks.get_mut(*i).unwrap();
        task.duration += active_time;
        task.active = keep_active;
        Ok(())
    }

    fn add_task<T: ToString + AsRef<str>>(&mut self, name: T, active: bool) {
        match self.names.get(name.as_ref()) {
            Some(&i) => {
                let task = self.tasks.get_mut(i).unwrap();
                task.active = active;
            }
            None => {
                self.names.insert(name.to_string(), self.tasks.len());
                self.tasks.push(Task {
                    name: name.to_string(),
                    duration: Duration::from_secs(0),
                    active,
                });
            }
        };
    }

    fn rename_task(&mut self, to: &String, from: &String) -> Result<(), String> {
        let i = self
            .names
            .remove(from.as_str())
            .ok_or(format!("Couldn't find task name '{}' while renaming", from))?;
        self.names.insert(to.to_owned(), i);
        self.tasks.get_mut(i).unwrap().name = to.to_owned();
        Ok(())
    }

    fn add_work_time(&mut self, from: DateTime<FixedOffset>, to: DateTime<FixedOffset>) {
        self.work_duration += to.sub(from).to_std().unwrap();
        if let Some((_, last_to)) = self.work_times.last_mut() {
            if from == *last_to {
                *last_to = to;
                return;
            }
        }
        self.work_times.push((from, to));
    }
}
