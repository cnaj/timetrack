use crate::taskregistry::State::{DayTracking, Idle, TaskActive};
use crate::timelog::{LogEvent, TimelogEntry};
use chrono::{DateTime, FixedOffset};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Task {
    name: String,
    duration: Duration,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum State {
    Idle,
    DayTracking,
    TaskActive,
}

struct TaskRegistryBuilder {
    tasks: Vec<Task>,
    names: HashMap<String, usize>,
    start_time: Option<DateTime<FixedOffset>>,
    state: State,
    current_task_name: Option<String>,
}

impl TaskRegistryBuilder {
    fn add_entry(&mut self, entry: &TimelogEntry) -> Result<(), String> {
        self.state = match self.state {
            Idle => match &entry.event {
                LogEvent::On => {
                    self.tasks.clear();
                    self.names.clear();
                    self.start_task(&entry.time, "Pause");
                    self.start_task(&entry.time, "n/n");
                    DayTracking
                }
                LogEvent::Continue => {
                    self.add_time_to_task("Pause", &entry.time)?;
                    self.start_time = Some(entry.time);
                    DayTracking
                }
                LogEvent::Start(name) => {
                    self.add_time_to_task("Pause", &entry.time)?;
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
                    self.add_time_to_task("n/n", &entry.time)?;
                    self.start_time = Some(entry.time);
                    Idle
                }
                LogEvent::Start(name) => {
                    self.add_time_to_task("n/n", &entry.time)?;
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
                    self.add_time_to_current_task(&entry.time)?;
                    self.start_time = Some(entry.time);
                    DayTracking
                }
                LogEvent::Off => {
                    self.add_time_to_current_task(&entry.time)?;
                    self.start_time = Some(entry.time);
                    Idle
                }
                LogEvent::Start(name) => {
                    self.add_time_to_current_task(&entry.time)?;
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
        };

        Ok(())
    }

    fn add_time_to_current_task(&mut self, time: &DateTime<FixedOffset>) -> Result<(), String> {
        let name = self
            .current_task_name
            .as_ref()
            .ok_or(format!("No current task recorded in state {:?}", self.state))?
            .to_owned();
        self.add_time_to_task(&name, time)?;
        Ok(())
    }

    fn add_time_to_task(&mut self, name: &str, time: &DateTime<FixedOffset>) -> Result<(), String> {
        let time_diff = *time
            - self
                .start_time
                .ok_or(format!("Invalid state: No start time recorded"))?;
        let duration: Duration = time_diff
            .to_std()
            .map_err(|e| format!("Non-continuous timestamp: {}", e))?;
        let i = self
            .names
            .get(name)
            .ok_or(format!("Couldn't find task name '{}'", name))?;
        self.tasks.get_mut(*i).unwrap().duration += duration;
        Ok(())
    }

    fn start_task<T: ToString + AsRef<str>>(
        &mut self,
        time: &DateTime<FixedOffset>,
        name: T,
    ) -> () {
        self.start_time = Some(*time);
        if !self.names.contains_key(name.as_ref()) {
            self.names.insert(name.to_string(), self.tasks.len());
            self.tasks.push(Task {
                name: name.to_string(),
                duration: Duration::from_secs(0),
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskRegistry {
    tasks: Vec<Task>,
    names: HashMap<String, usize>,
}

impl TaskRegistry {
    pub fn build<I: Iterator<Item = TimelogEntry>>(entries: I) -> Result<TaskRegistry, String> {
        let mut builder = TaskRegistryBuilder {
            tasks: Vec::new(),
            names: HashMap::new(),
            start_time: None,
            state: Idle,
            current_task_name: None,
        };

        for entry in entries {
            builder.add_entry(&entry)?;
        }

        Ok(TaskRegistry {
            tasks: builder.tasks,
            names: builder.names,
        })
    }

    pub fn get_tasks(&self) -> &[Task] {
        self.tasks.as_slice()
    }
}
