//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::config::BIG_STRIDE;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// sort the ready_queue
    fn sort(&mut self) {
        self.ready_queue.make_contiguous().sort_by(|a,b| {
            let a_inner = a.inner_exclusive_access();
            let b_inner = b.inner_exclusive_access();
            a_inner.stride.cmp(&b_inner.stride)
        });
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        let mut task_inner = task.inner_exclusive_access();
        let pass = BIG_STRIDE / (task_inner.priority as usize);
        if task_inner.stride.wrapping_add(pass) <= task_inner.stride {
            self.sort();
            if let Some(first) = self.ready_queue.front() {
                let stride_min = first.inner_exclusive_access().stride;
                let stride_min = stride_min.min(task_inner.stride);
                for tcp in self.ready_queue.iter_mut() {
                    tcp.inner_exclusive_access().stride -= stride_min;
                }
                task_inner.stride = task_inner.stride - stride_min + pass;
            } else {
                task_inner.stride = 0;
            }
        } else {
            task_inner.stride += pass;
        }
        drop(task_inner);
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.sort();
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
