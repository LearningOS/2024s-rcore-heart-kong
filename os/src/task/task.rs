//! Types related to task management

use crate::{config::MAX_SYSCALL_NUM, timer::get_time_ms};

use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The task syscall times
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// The duration of syscall and task
    pub time: usize,
    /// The time of starting task
    pub start_time: usize
}

impl TaskControlBlock {
    /// Update the duration of syscall
    pub fn update_time(&mut self) {
        if self.start_time == 0 {
            self.start_time = get_time_ms();
        } else {
            let cur_time = get_time_ms();
            self.time = cur_time - self.start_time;
        }
    }
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
