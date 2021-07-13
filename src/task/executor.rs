use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Waker, Poll, Context};
use crossbeam_queue::ArrayQueue;
use spin::Mutex;

lazy_static::lazy_static! {
    pub static ref EXECUTOR: Mutex<Executor> = Mutex::new(Executor::new());
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
    currently_running: Option<TaskId>,
}

unsafe impl Send for Executor {}
unsafe impl Sync for Executor {}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
            currently_running: None,
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let id = task.id;
        if self.tasks.insert(id, task).is_some() {
            panic!("The same task was launched twice");
        }
        self.task_queue.push(id).expect("Task queue is full");
    }

    pub fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
            currently_running
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut ctx = Context::from_waker(waker);
            *currently_running = Some(task_id);
            crate::println!("running task {}", task_id.0);
            match task.poll(&mut ctx) {
                Poll::Ready(_) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                },
                Poll::Pending => {},
            }
            *currently_running = None;
        }
    }

    pub fn run(&mut self) -> ! {
        crate::println!("exec.run");
        loop {
            crate::println!("loop");
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    pub fn requeue_current_task(&mut self) {
        if let Some(current_task) = self.currently_running {
            if let Some(waker) = self.waker_cache.get(&current_task) {
                waker.wake_by_ref();
            }
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts;

        interrupts::disable();
        if self.task_queue.is_empty() {
            interrupts::enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        crate::println!("waking {}", self.task_id.0);
        self.task_queue.push(self.task_id).expect("TaskWaker's queue is full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
