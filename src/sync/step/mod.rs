use std::thread;
use std::time::SystemTime;
use log::info;
use crate::core::job::now_time;
use crate::core::step::{mount_step_status, StepStatus, throw_tolerant_exception};

pub mod complex_step;
pub mod simple_step;
pub mod step_builder;

/// A trait for objects that can be executed.
pub trait Runner where Self: Sized {
    /// The type of output produced by the execution.
    type Output;

    /// Executes the object and returns the output.
    fn run(self) -> Self::Output;
}

/// A trait for objects that can make decisions.
pub trait Decider {
    /// Checks if the object should be executed.
    fn is_run(&self) -> bool;
}

pub type StepCallback = Box<dyn FnOnce() + Send>;

pub type DeciderCallback = Box<dyn Fn() -> bool>;

/// Represents a synchronous step in a job.
pub struct SyncStep {
    /// The start time of the step execution.
    #[allow(dead_code)]
    pub(crate) start_time: Option<u64>,
    /// The end time of the step execution.
    #[allow(dead_code)]
    pub(crate) end_time: Option<u64>,
    /// The name of the step.
    pub name: String,
    /// Indicates whether the step is tolerant to thrown exceptions.
    pub throw_tolerant: Option<bool>,
    /// The decider callback for the step.
    pub(crate) decider: Option<DeciderCallback>,
    /// The callback function to be executed as the step.
    pub(crate) callback: Option<Box<dyn FnOnce() -> () + Send>>,
}

impl Runner for SyncStep {
    /// The output type of the step execution.
    type Output = StepStatus;

    /// Executes the step and returns its status.
    fn run(self) -> Self::Output {
        return match self.callback {
            None => {
                throw_tolerant_exception(self.throw_tolerant.unwrap_or(false), self.name)
            }
            Some(callback) => {
                info!("Step {} is running", self.name);
                let task = thread::spawn(move || {
                    callback();
                });
                let task_result = task.join();

                let start_time = now_time();
                return match task_result {
                    Ok(_) => {
                        let message = format!("Step {} executed successfully", self.name);
                        info!("{}", message);
                        mount_step_status(Ok(message), start_time)
                    },
                    Err(_) => {
                        let message = format!("Step {} failed to execute", self.name);
                        info!("{}", message);
                        mount_step_status(Err(message), start_time)
                    },
                };
            }
        };
    }
}

impl Decider for SyncStep {
    /// Checks if the step should be executed based on the decider callback.
    fn is_run(&self) -> bool {
        return match &self.decider {
            None => true,
            Some(decider) => decider(),
        };
    }
}

// Allows `SyncStep` to be sent between threads safely.
unsafe impl Send for SyncStep {}