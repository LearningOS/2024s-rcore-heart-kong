use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    let id_: usize;
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id_ = id;
    } else {
        process_inner.mutex_list.push(mutex);
        id_ = process_inner.mutex_list.len() - 1;
    }
    process_inner.mutex_available[id_] += 1;

    id_ as isize
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();

    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();

    task_inner.mutex_demand[mutex_id] += 1;

    drop(task_inner);
    drop(task);

    // deadlock detect
    if process_inner.lock_detect{
        let mut mutex_available = process_inner.mutex_available.clone();
        let mut finish = vec![false; process_inner.tasks.len()];
        loop{
            let mut found = false;
            for i in 0..process_inner.tasks.len(){
                if finish[i]==false{
                    let task = process_inner.get_task(i);
                    let task_inner = task.inner_exclusive_access();
                    let undemand = mutex_available.iter().enumerate().any(|(mux, num)|{
                        task_inner.mutex_demand[mux] > *num
                    });
                    if !undemand{
                        finish[i] = true;
                        mutex_available.iter_mut().enumerate().for_each(|(pos, ptr)|{
                            *ptr += task_inner.mutex_allot[pos];
                        });
                        found=true;
                    }
                    drop(task_inner);
                    drop(task);
                }
            }
            if !found{
                break;
            }
        }
        if finish.iter().any(|&x| x==false){
            let task = current_task().unwrap();
            let mut task_inner = task.inner_exclusive_access();
            task_inner.mutex_demand[mutex_id] -= 1;
            return -0xDEAD;
        }
    }

    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    process_inner.mutex_available[mutex_id] -= 1;
    task_inner.mutex_demand[mutex_id] -= 1;
    task_inner.mutex_allot[mutex_id] += 1;
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    drop(task_inner);
    drop(task);
    mutex.lock();
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    process_inner.mutex_available[mutex_id] += 1;
    task_inner.mutex_allot[mutex_id] -= 1;
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    drop(task_inner);
    drop(task);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    process_inner.sem_available[id] += res_count;
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    process_inner.sem_available[sem_id] += 1;
    task_inner.sem_allot[sem_id] -= 1;
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    drop(task_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.sem_need[sem_id] += 1;
    drop(task_inner);
    drop(task);

    if process_inner.lock_detect{
        let mut mutex_available = process_inner.sem_available.clone();
        let mut finish = vec![false;process_inner.tasks.len()];
        loop{
            let mut found = false;
            for i in 0..process_inner.tasks.len(){
                if finish[i]==false{
                    let task = process_inner.get_task(i);
                    let task_inner = task.inner_exclusive_access();
                    let undemand = mutex_available.iter().enumerate().any(|(mux, num)|{
                        task_inner.sem_need[mux] > *num
                    });
                    if !undemand{
                        finish[i] = true;
                        mutex_available.iter_mut().enumerate().for_each(|(pos,num)|{
                            *num += task_inner.sem_allot[pos];
                        });
                        found = true;
                    }
                    drop(task_inner);
                    drop(task);
                }
            }
            if !found{
                break;
            }

        }

        if finish.iter().any(|&x| x==false){
            let task = current_task().unwrap();
            let mut task_inner = task.inner_exclusive_access();
            task_inner.mutex_demand[sem_id] -= 1;
            return -0xDEAD;
        }

    }    
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();

    if process_inner.sem_available[sem_id] >0 {
        task_inner.sem_need[sem_id] -= 1;
        task_inner.sem_allot[sem_id] += 1;
        process_inner.sem_available[sem_id] -= 1;
    }

    drop(task_inner);
    drop(task);
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down();
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    if enabled==1{
        current_process().inner_exclusive_access().lock_detect=true;
        return 0;
    }else if enabled==0{
        current_process().inner_exclusive_access().lock_detect=false;
        return 0;
    }
    -1
}
