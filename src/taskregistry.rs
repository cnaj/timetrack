use crate::timelog::{LogEvent, TimelogEntry};
use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, FixedOffset};
use crate::taskregistry::State::{Idle, DayTracking, TaskActive};

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
}

impl TaskRegistryBuilder {
    fn add_entry(&mut self, entry: &TimelogEntry) -> Result<(), String> {
        self.state = match self.state {
            Idle => {
                match &entry.event {
                    LogEvent::On => {
                        self.tasks.clear();
                        self.names.clear();
                        self.start_time = None;
                        DayTracking
                    }
                    LogEvent::Continue => {
                        DayTracking
                    }
                    LogEvent::Start(name) => {
                        self.start_task(entry, name);
                        TaskActive
                    }
                    _ => {
                        return Err(format!("Invalid event {:?} in state {:?}", entry.event, self.state));
                    }
                }
            }
            DayTracking => {
                return Err(format!("not implemented"));
            }
            TaskActive => {
                return Err(format!("not implemented"));
            }
        };

        Ok(())
    }

    fn start_task(&mut self, entry: &TimelogEntry, name: &String) -> () {
        self.start_time = Some(entry.time);
        if !self.names.contains_key(name) {
            self.names.insert(name.to_owned(), self.tasks.len());
            self.tasks.push(Task { name: name.to_owned(), duration: Duration::from_secs(0) });
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskRegistry {
    tasks: Vec<Task>,
    names: HashMap<String, usize>,
}

impl TaskRegistry {
    pub fn build<I: Iterator<Item=TimelogEntry>>(entries: I) -> Result<TaskRegistry, String> {
        let mut builder = TaskRegistryBuilder {
            tasks: Vec::new(),
            names: HashMap::new(),
            start_time: None,
            state: Idle,
        };

        for entry in entries {
            builder.add_entry(&entry)?;
        }

        Ok(TaskRegistry { tasks: builder.tasks, names: builder.names })
    }

    pub fn get_tasks(&self) -> &[Task] {
        self.tasks.as_slice()
    }
}
