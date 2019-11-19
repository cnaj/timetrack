use crate::timelog::{LogEvent, TimelogEntry};
use std::collections::HashMap;

#[derive(Debug)]
struct Task {
    name: String,
}

#[derive(Debug)]
pub struct TaskRegistry {
    tasks: Vec<Task>,
    names: HashMap<String, usize>,
}

impl TaskRegistry {
    fn new(tasks: Vec<Task>) -> TaskRegistry {
        let mut names: HashMap<String, usize> = HashMap::new();

        for task in &tasks {
            names.insert(task.name.to_owned(), 0);
        }

        TaskRegistry { tasks, names }
    }

    fn build<I: Iterator<Item = TimelogEntry>>(entries: I) -> Result<TaskRegistry, String> {
        let mut tasks: Vec<Task> = Vec::new();
        let mut names: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            match entry.event {
                LogEvent::Start(name) => {
                    if !names.contains_key(&name) {
                        names.insert(name.to_owned(), tasks.len());
                        tasks.push(Task { name });
                    }
                }
                _ => {}
            }
        }

        Ok(TaskRegistry { tasks, names })
    }
}
