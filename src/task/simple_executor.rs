use super::Task;
use alloc::collections::VecDeque;
use core::task::{RawWaker, RawWakerVTable, Waker, Context, Poll};

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>, // simple FIFO queue
}

impl SimpleExecutor {
    pub fn new() -> Self {
        SimpleExecutor {
            task_queue: VecDeque::new()
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task), // add it to the back
            }
        }
    }
}

// ultra minimalistic raw waker, which does litteraly nothing
fn dummy_raw_waker() -> RawWaker { // requires defining vtable
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}