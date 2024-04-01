use std::thread;
use std::time::SystemTime;

use crate::core::step::{mount_step_status, StepStatus, throw_tolerant_exception};

pub mod complex_step;
pub mod simple_step;
pub mod step_builder;

pub trait Runner where Self: Sized {
    type Output;
    fn run(self) -> Self::Output;
}

pub trait Decider {
    fn is_run(&self) -> bool;
}

pub type StepCallback = Box<dyn FnOnce() + Send>;

pub type DeciderCallback = Box<dyn Fn() -> bool>;

pub struct SyncStep {
    #[allow(dead_code)]
    pub(crate) start_time: Option<u64>,
    #[allow(dead_code)]
    pub(crate) end_time: Option<u64>,
    pub name: String,
    pub throw_tolerant: Option<bool>,
    pub(crate) decider: Option<DeciderCallback>,
    pub(crate) callback: Option<Box<dyn FnOnce() -> () + Send>>,
}

impl Runner for SyncStep {
    type Output = StepStatus;
    fn run(self) -> Self::Output {
        let start_time = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();

        return match self.callback {
            None => {
                throw_tolerant_exception(self.throw_tolerant.unwrap_or(false), self.name)
            }
            Some(callback) => {
                let thread = thread::spawn(move || {
                    callback();
                });
                let thread_result = thread.join();

                return match thread_result {
                    Ok(_) => mount_step_status(Ok(format!("Step {} executed successfully", self.name)), start_time),
                    Err(_) => mount_step_status(Err(format!("Step {} failed to execute", self.name)), start_time),
                };
            }
        };
    }
}

impl Decider for SyncStep {
    fn is_run (&self) -> bool {
        return match &self.decider {
            None => true,
            Some(decider) => decider(),
        };
    }
}

unsafe impl Send for SyncStep {}