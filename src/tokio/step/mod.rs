use async_trait::async_trait;
use futures::future::BoxFuture;
use log::info;
use crate::core::job::now_time;
use crate::core::step::{mount_step_status, StepStatus, throw_tolerant_exception};

pub mod simple_step;
pub mod step_builder;
pub mod complex_step;
pub mod parallel_step_builder;

/// A trait for running asynchronous tasks and returning a result.
#[async_trait]
pub trait AsyncStepRunner<R> where Self: Sized + Send {
    /// Executes the asynchronous task and returns the result.
    async fn run(self) -> R;
}

/// A trait representing a decider for asynchronous steps.
#[async_trait]
pub trait Decider {
    /// Decides whether the step should proceed or not.
    async fn decide(&self) -> bool;
}

/// Type alias for a dynamic asynchronous callback function.
pub type DynAsyncCallback<O> = dyn Send + Sync + Fn() -> BoxFuture<'static, O>;

/// Type alias for a decider callback function.
pub type DeciderCallback = Box<dyn Send + Sync + Fn() -> BoxFuture<'static, bool>>;

/// Represents an asynchronous step with configurable callbacks and deciders.
pub struct AsyncStep {
    /// The name of the step.
    pub name: String,
    /// Whether the step is tolerant to thrown errors.
    pub throw_tolerant: Option<bool>,
    /// The decider callback for the step.
    decider: Option<DeciderCallback>,
    /// The callback function for the step.
    callback: Option<Box<DynAsyncCallback<()>>>,
}

#[async_trait]
impl AsyncStepRunner<StepStatus> for AsyncStep {
    /// Executes the asynchronous step and returns its status.
    async fn run(self) -> StepStatus {
        return match self.callback {
            None => {
                throw_tolerant_exception(self.throw_tolerant.unwrap_or(false), self.name)
            }
            Some(callback) => {
                let start_time = now_time();
                info!("Step {} is running", self.name);
                let callback_result = tokio::spawn(async move {
                    callback().await;
                }).await;
                return match callback_result {
                    Ok(_) => {
                        let message = format!("Step {} executed successfully", self.name);
                        info!("{}", message);
                        mount_step_status(self.name, Ok(message), start_time)
                    },
                    Err(_) => {
                        let message = format!("Step {} failed to execute", self.name);
                        info!("{}", message);
                        mount_step_status(self.name, Err(message), start_time)
                    },
                };
            }
        };
    }
}

#[async_trait]
impl Decider for AsyncStep {
    /// Decides whether the step should proceed or not based on the decider callback.
    async fn decide(&self) -> bool {
        return match &self.decider {
            None => true,
            Some(decider) => decider().await,
        };
    }
}

// Allows `AsyncStep` to be sent between threads safely.
unsafe impl Send for AsyncStep {}