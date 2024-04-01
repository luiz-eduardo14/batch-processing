use std::time::SystemTime;

use async_trait::async_trait;
use futures::future::BoxFuture;
use crate::core::step::StepStatus;

pub mod simple_step;
pub mod step_builder;
pub mod complex_step;

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
    callback: Option<Box<DynAsyncCallback<Result<String, String>>>>,
}

#[async_trait]
impl AsyncStepRunner<StepStatus> for AsyncStep {
    /// Executes the asynchronous step and returns its status.
    async fn run(self) -> StepStatus {
        let start_time = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
        return match self.callback {
            None => {
                if self.throw_tolerant.unwrap_or(false) {
                    return StepStatus {
                        start_time: None,
                        end_time: None,
                        status: Ok(String::from("callback is required, please provide a callback to the step")),
                    }
                }
                panic!("callback is required, please provide a callback to the step with name: {}", self.name)
            }
            Some(callback) => {
                let result = callback().await;
                let end_time = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();

                match result {
                    Ok(message) => StepStatus {
                        start_time: Some(start_time),
                        end_time: Some(end_time),
                        status: Ok(message),
                    },
                    Err(message) => StepStatus {
                        start_time: Some(start_time),
                        end_time: None,
                        status: Err(message),
                    },
                }
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