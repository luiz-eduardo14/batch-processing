// pub mod complex_step;
// pub mod simple_step;
// pub mod step_builder;

pub mod simple_step;
pub mod step_builder;

use std::sync::Arc;
use async_trait::async_trait;
use futures::future::BoxFuture;

#[async_trait]
pub trait AsyncRunner where Self: Sized + Send {
    async fn run(self) -> Result<String, String>;
}

type DynAsyncCallback<I, O> = dyn 'static + Send + Sync + Fn(I) -> BoxFuture<'static, O>;

type DeciderCallback = fn() -> bool;

pub struct AsyncStep<C: 'static> {
    context: Arc<C>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    pub name: String,
    pub throw_tolerant: Option<bool>,
    decider: Option<DeciderCallback>,
    callback: Option<Box<DynAsyncCallback<Arc<C>, Result<String, String>>>>,
}

#[async_trait]
impl <C: Send + Sync> AsyncRunner for AsyncStep<C> {
    async fn run(self) -> Result<String, String> {
        return match self.callback {
            None => {
                if self.throw_tolerant.unwrap_or(false) {
                    return Err(format!("callback is required, please provide a callback to the step with name: {}", self.name));
                }
                panic!("callback is required, please provide a callback to the step with name: {}", self.name)
            }
            Some(callback) => {
                return callback(self.context).await
            }
        };
    }
}

unsafe impl <C> Send for AsyncStep<C> {}