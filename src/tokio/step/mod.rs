// pub mod complex_step;
// pub mod simple_step;
// pub mod step_builder;

pub mod simple_step;
pub mod step_builder;

use std::sync::Arc;
use std::time::SystemTime;
use async_trait::async_trait;
use futures::future::BoxFuture;

#[async_trait]
pub trait AsyncRunner<C: 'static, R> where Self: Sized + Send {
    async fn run(self, context: Arc<C>) -> R;
}

type DynAsyncCallback<I, O> = dyn 'static + Send + Sync + Fn(I) -> BoxFuture<'static, O>;

type DeciderCallback = fn() -> bool;

#[derive(Debug, Clone)]
pub struct StepStatus {
    start_time: Option<u128>,
    end_time: Option<u128>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum StepResult {
    Success(StepStatus),
    Failure(StepStatus),
}

pub struct AsyncStep<C: 'static> {
    pub name: String,
    pub throw_tolerant: Option<bool>,
    decider: Option<DeciderCallback>,
    callback: Option<Box<DynAsyncCallback<Arc<C>, Result<String, String>>>>,
}

#[async_trait]
impl <C: Send + Sync> AsyncRunner<C, StepResult> for AsyncStep<C> {
    async fn run(self, context: Arc<C>) -> StepResult {
        return match self.callback {
            None => {
                if self.throw_tolerant.unwrap_or(false) {
                    return StepResult::Failure(StepStatus {
                        start_time: None,
                        end_time: None,
                        status: String::from("callback is required, please provide a callback to the step"),
                    });
                }
                panic!("callback is required, please provide a callback to the step with name: {}", self.name)
            }
            Some(callback) => {
                let start_time = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                let result = callback(context).await;
                let end_time = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();

                match result {
                    Ok(message) => StepResult::Success(StepStatus {
                        start_time: Some(start_time),
                        end_time: Some(end_time),
                        status: message,
                    }),
                    Err(message) => StepResult::Failure(StepStatus {
                        start_time: Some(start_time),
                        end_time: Some(end_time),
                        status: message,
                    }),
                }
            }
        };
    }
}

unsafe impl <C> Send for AsyncStep<C> {}