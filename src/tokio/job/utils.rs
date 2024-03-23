use std::sync::Arc;
use futures::lock::{Mutex, MutexGuard};
use log::{error, info};
use tokio::task::{AbortHandle, JoinSet};
use crate::tokio::step::{AsyncRunner, AsyncStep, StepResult};

pub fn log_step(message: Result<String, String>) {
    match message {
        Ok(message) => {
            info!("{}", message);
        }
        Err(message) => {
            error!("{}", message);
        }
    }
}

pub async fn run_all_join_handles(join_set: Arc<Mutex<JoinSet<StepResult>>>) -> Vec<StepResult> {
    let join_set = Arc::clone(&join_set);
    let mut join_set = join_set.lock().await;
    let mut is_panicked = false;
    let mut step_results: Vec<StepResult> = Vec::new();
    while let Some(join_handle) = join_set.join_next().await {
        match join_handle {
            Ok(message) => {
                step_results.push(message.clone());
                match message {
                    StepResult::Success(status) => {
                        log_step(Ok(status.status));
                    }
                    StepResult::Failure(status) => {
                        log_step(Err(status.status));
                    }

                }
            }
            Err(join_error) => {
                if join_error.is_panic() {
                    is_panicked = true;
                    break;
                } else {
                    log_step(Err(format!("Join error: {:?}", join_error)));
                }
            }
        };
    }

    if is_panicked {
        join_set.abort_all();
    }

    return step_results;
}

pub async fn mount_step_task<C: 'static + Sync + Send>(step: AsyncStep<C>, context: Arc<C>, throw_tolerant: bool, mut join_set: MutexGuard<'_, JoinSet<StepResult>>) -> AbortHandle {
    return join_set.spawn(async move {
        return match step.run(context).await {
            StepResult::Success(status) => {
                log_step(Ok(status.status.clone()));
                StepResult::Success(
                    status
                )
            }
            StepResult::Failure(status) => {
                if throw_tolerant {
                    panic!("{}", status.status);
                } else {
                    log_step(Err(status.status.clone()));
                    StepResult::Failure(
                        status
                    )
                }
            }
        };
    });
}