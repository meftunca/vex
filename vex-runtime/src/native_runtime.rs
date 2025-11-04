//! Vex Native Async Runtime
//!
//! A lightweight, self-contained async runtime for Vex programs.
//! No external dependencies like tokio - pure Vex/Rust implementation.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Task state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Suspended,
    Completed,
}

/// A lightweight task (coroutine)
pub struct Task {
    pub id: usize,
    pub state: TaskState,
    // TODO: Stack or state machine data
}

/// Simple task scheduler
pub struct Scheduler {
    tasks: VecDeque<Task>,
    next_id: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            next_id: 0,
        }
    }

    /// Spawn a new task
    pub fn spawn(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let task = Task {
            id,
            state: TaskState::Ready,
        };

        self.tasks.push_back(task);
        id
    }

    /// Run the scheduler (simple round-robin)
    pub fn run(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            match task.state {
                TaskState::Ready => {
                    task.state = TaskState::Running;
                    // Execute task (TODO: actual execution)
                    task.state = TaskState::Completed;
                }
                TaskState::Suspended => {
                    // Put back in queue
                    self.tasks.push_back(task);
                }
                TaskState::Completed => {
                    // Task done, drop it
                }
                TaskState::Running => {
                    unreachable!("Task should not be in Running state in queue");
                }
            }
        }
    }

    /// Yield current task (suspend and reschedule)
    pub fn yield_now(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.state = TaskState::Suspended;
        }
    }
}

/// Global runtime instance
static RUNTIME: Mutex<Option<Scheduler>> = Mutex::new(None);

/// Initialize the native runtime
#[unsafe(no_mangle)]
pub extern "C" fn vex_native_runtime_init() {
    let mut rt = RUNTIME.lock().unwrap();
    *rt = Some(Scheduler::new());
}

/// Spawn a new task in native runtime
#[unsafe(no_mangle)]
pub extern "C" fn vex_native_runtime_spawn() -> usize {
    let mut rt = RUNTIME.lock().unwrap();
    rt.as_mut().unwrap().spawn()
}

/// Run the native runtime (blocking)
#[unsafe(no_mangle)]
pub extern "C" fn vex_native_runtime_run() {
    let mut rt = RUNTIME.lock().unwrap();
    rt.as_mut().unwrap().run();
}

/// Yield current task in native runtime
#[unsafe(no_mangle)]
pub extern "C" fn vex_native_runtime_yield(task_id: usize) {
    let mut rt = RUNTIME.lock().unwrap();
    rt.as_mut().unwrap().yield_now(task_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler() {
        let mut scheduler = Scheduler::new();
        let task1 = scheduler.spawn();
        let task2 = scheduler.spawn();

        assert_eq!(task1, 0);
        assert_eq!(task2, 1);
        assert_eq!(scheduler.tasks.len(), 2);
    }
}
