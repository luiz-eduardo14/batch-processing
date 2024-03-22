use std::sync::Arc;
use futures::lock::{Mutex, MutexGuard};
use futures::task::SpawnExt;
use log::{error, info};
use tokio::task::{AbortHandle, JoinSet};
use crate::sync::step::{Runner, Step};
use crate::tokio::step::{AsyncRunner, AsyncStep};

pub fn log_step(result: Result<String, String>) {
    match result {
        Ok(success_message) => {
            info!("{}", success_message);
        }
        Err(error_message) => {
            error!("{}", error_message);
        }
    }
}

pub async fn run_all_join_handles(join_set: Arc<Mutex<JoinSet<Result<String, String>>>>) {
    let join_set = Arc::clone(&join_set);
    let mut join_set = join_set.lock().await;
    let mut is_panicked = false;
    while let Some(join_handle) = join_set.join_next().await {
        match join_handle {
            Ok(message) => {
                log_step(message);
            }
            Err(join_error) => {
                if join_error.is_panic() {
                    is_panicked = true;
                } else {
                    log_step(Err(join_error.to_string()));
                }
            }
        };
    }

    if is_panicked {
        join_set.abort_all();
    }
}

pub async fn mount_step_task<C: 'static + Sync + Send>(step: AsyncStep<C>, context: Arc<C>, throw_tolerant: bool, mut join_set: MutexGuard<'_, JoinSet<Result<String, String>>>) -> AbortHandle {
    return join_set.spawn(async move {
        return match step.run(context).await {
            Ok(success_message) => {
                log_step(Ok(success_message.clone()));
                Ok(success_message)
            }
            Err(error_message) => {
                if throw_tolerant {
                    panic!("{}", error_message);
                } else {
                    log_step(Err(error_message.clone()));
                    Err(error_message)
                }
            }
        };
    });
}