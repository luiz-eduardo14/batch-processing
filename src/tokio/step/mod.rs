// pub mod complex_step;
// pub mod simple_step;
// pub mod step_builder;

use std::time::SystemTime;

use async_trait::async_trait;
use futures::future::BoxFuture;

pub mod simple_step;
pub mod step_builder;
pub mod complex_step;

#[async_trait]
pub trait AsyncRunner<R> where Self: Sized + Send {
    async fn run(self) -> R;
}

#[async_trait]
pub trait Decider {
    async fn decide(&self) -> bool;
}

pub type DynAsyncCallback<O> = dyn Send + Sync + Fn() -> BoxFuture<'static, O>;

pub type DeciderCallback = Box<dyn Send + Sync + Fn() -> BoxFuture<'static, bool>>;

#[derive(Debug, Clone)]
pub struct StepStatus {
    #[allow(dead_code)]
    pub start_time: Option<u128>,
    #[allow(dead_code)]
    pub end_time: Option<u128>,
    pub status: Result<String, String>,
}

pub struct AsyncStep<> {
    pub name: String,
    pub throw_tolerant: Option<bool>,
    decider: Option<DeciderCallback>,
    callback: Option<Box<DynAsyncCallback<Result<String, String>>>>,
}

#[async_trait]
impl AsyncRunner<StepStatus> for AsyncStep {
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
    async fn decide(&self) -> bool {
        return match &self.decider {
            None => {
                true
            }
            Some(decider) => {
                return decider().await
            }
        };
    }
}

unsafe impl Send for AsyncStep {}