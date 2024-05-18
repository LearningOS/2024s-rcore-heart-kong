//! Process management syscalls
use crate::{
    config::{CLOCK_FREQ, MAX_SYSCALL_NUM}, mm::{get_pa_by_va, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, free_space_current_task, get_currnet_task_info, malloc_space_current_task, suspend_current_and_run_next, TaskStatus
    }, timer::get_time
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

impl TaskInfo {
    /// update
    pub fn update(&mut self, status: TaskStatus, syscall_times: [u32; MAX_SYSCALL_NUM], time: usize) {
        self.status = status;
        self.syscall_times = syscall_times;
        self.time = time;
    }
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let ppn = get_pa_by_va(current_user_token(), ts as *const _ as usize);
    let ts: &mut TimeVal = ppn.get_mut();
    let time = get_time();
    ts.sec = time / CLOCK_FREQ;
    ts.usec = time * 1_000_000 / CLOCK_FREQ;
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let ppn = get_pa_by_va(current_user_token(), ti as *const _ as usize);
    let ti: &mut TaskInfo = ppn.get_mut();
    get_currnet_task_info(ti);
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let start_va = VirtAddr::from(start);
    if (start_va.page_offset()) != 0 {
        return -1
    }
    let end_va = VirtAddr::from(start + len);
    if (port >> 3) != 0 || port == 0 {
        return -1
    }
    let port = port << 1;
    malloc_space_current_task(start_va.into(), end_va.into(), port as u8)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let start_va = VirtAddr::from(start);
    if (start_va.page_offset()) != 0 {
        return -1
    }
    let end_va = VirtAddr::from(start + len);
    free_space_current_task(start_va.into(), end_va.into())
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
