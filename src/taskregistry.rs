use std::collections::HashMap;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::mem::replace;
use std::ops::Sub;
use std::time::Duration;

use chrono::{DateTime, FixedOffset, Local, Timelike};

use crate::taskregistry::State::{DayTracking, Idle, TaskActive};
use crate::timelog::{LogEvent, TimelogEntry};

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Task {
    pub name: String,
    pub duration: Duration,
}

impl Task {
    pub fn new(name: impl ToString, duration_mins: u64) -> Task {
        Task {
            name: name.to_string(),
            duration: Duration::from_secs(duration_mins * 60),
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let secs = self.duration.as_secs();
        let mins = secs / 60;
        let m = mins % 60;
        let h = mins / 60;
        write!(f, "{:02}:{:02}\t{}", h, m, self.name)
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
                    self.start_task(&entry.time, "Pause");
                    self.start_task(&entry.time, "n/n");
                    DayTracking
                }
                LogEvent::Continue => {
                    self.start_work_time(entry);
                    self.add_time_to_task("Pause", &entry.time)?;
                    self.start_time = Some(entry.time);
                    DayTracking
                }
                LogEvent::Start(name) => {
                    self.start_work_time(entry);
                    self.add_time_to_task("Pause", &entry.time)?;
                    self.start_task(&entry.time, name);
                    self.current_task_name = Some(name.clone());
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
                    self.add_time_to_task("n/n", &entry.time)?;
                    self.start_time = Some(entry.time);
                    self.stop_work_time(entry);
                    Idle
                }
                LogEvent::Start(name) => {
                    self.add_time_to_task("n/n", &entry.time)?;
                    self.start_task(&entry.time, name);
                    self.current_task_name = Some(name.clone());
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
                    self.add_time_to_current_task(&entry.time)?;
                    self.current_task_name = None;
                    self.start_time = Some(entry.time);
                    DayTracking
                }
                LogEvent::Off => {
                    self.add_time_to_current_task(&entry.time)?;
                    self.current_task_name = None;
                    self.start_time = Some(entry.time);
                    self.stop_work_time(entry);
                    Idle
                }
                LogEvent::Start(name) => {
                    self.add_time_to_current_task(&entry.time)?;
                    self.start_task(&entry.time, name);
                    self.current_task_name = Some(name.to_owned());
                    TaskActive
                }
                LogEvent::Rename { to, from } => {
                    let name = from
                        .as_ref()
                        .or(self.current_task_name.as_ref())
                        .ok_or(format!("No current task name set while renaming"))?;
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

    pub fn finish(&mut self) -> Option<TaskRegistry> {
        if self.state != Idle {
            let now = Local::now()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            self.add_entry(&TimelogEntry::new(&now.into(), LogEvent::Off))
                .unwrap();
        }

        if self.start_time.is_some() {
            self.start_time = None;
            Some(replace(&mut self.task_registry, TaskRegistry::new()))
        } else {
            None
        }
    }

    fn start_work_time(&mut self, entry: &TimelogEntry) {
        self.work_start_time = Some(entry.time);
    }

    fn stop_work_time(&mut self, entry: &TimelogEntry) {
        self.task_registry
            .add_work_time(self.work_start_time.unwrap(), entry.time);
        self.work_start_time = None;
    }

    fn add_time_to_current_task(&mut self, time: &DateTime<FixedOffset>) -> Result<(), String> {
        let name = self
            .current_task_name
            .as_ref()
            .ok_or(format!(
                "No current task recorded in state {:?}",
                self.state
            ))?
            .to_owned();
        self.add_time_to_task(&name, time)?;
        Ok(())
    }

    fn add_time_to_task(&mut self, name: &str, time: &DateTime<FixedOffset>) -> Result<(), String> {
        let time_diff = *time - self.start_time
            .ok_or(format!("Invalid state: No start time recorded"))?;
        let duration: Duration = time_diff
            .to_std()
            .map_err(|e| format!("Non-continuous timestamp: {}", e))?;
        self.task_registry.add_time_to_task(name, duration)
    }

    fn start_task<T: ToString + AsRef<str>>(
        &mut self,
        time: &DateTime<FixedOffset>,
        name: T,
    ) -> () {
        self.start_time = Some(*time);
        self.task_registry.add_task(name);
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

    fn add_time_to_task(&mut self, name: &str, time: Duration) -> Result<(), String> {
        let i = self
            .names
            .get(name)
            .ok_or(format!("Couldn't find task name '{}'", name))?;
        self.tasks.get_mut(*i).unwrap().duration += time;
        Ok(())
    }

    fn add_task<T: ToString + AsRef<str>>(&mut self, name: T) {
        if !self.names.contains_key(name.as_ref()) {
            self.names.insert(name.to_string(), self.tasks.len());
            self.tasks.push(Task {
                name: name.to_string(),
                duration: Duration::from_secs(0),
            });
        }
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
