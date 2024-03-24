use std::sync::Arc;

use futures::lock::{Mutex, MutexGuard};
use log::{error, info};
use tokio::task::{AbortHandle, JoinSet};

use crate::tokio::step::{AsyncRunner, AsyncStep, StepStatus};

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

pub async fn run_all_join_handles(join_set: Arc<Mutex<JoinSet<StepStatus>>>) -> Vec<StepStatus> {
    let join_set = Arc::clone(&join_set);
    let mut join_set = join_set.lock().await;
    let mut is_panicked = false;
    let mut step_results: Vec<StepStatus> = Vec::new();
    while let Some(join_handle) = join_set.join_next().await {
        match join_handle {
            Ok(step_status) => {
                step_results.push(step_status.clone());
                match step_status.status {
                    Ok(message) => {
                        log_step(Ok(message));
                    }
                    Err(message) => {
                        log_step(Err(message));
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

pub async fn mount_step_task(step: AsyncStep, throw_tolerant: bool, mut join_set: MutexGuard<'_, JoinSet<StepStatus>>) -> AbortHandle {
    return join_set.spawn(async move {
        let step_result = step.run().await;
        match step_result.status.clone() {
            Ok(message) => {
                log_step(Ok(message.clone()));
            }
            Err(message) => {
                if throw_tolerant {
                    panic!("{}", message);
                } else {
                    log_step(Err(message));
                }
            }
        };

        return step_result;
    });
}