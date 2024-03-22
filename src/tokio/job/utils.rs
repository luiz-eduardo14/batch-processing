use std::os::linux::raw::stat;
use std::sync::Arc;
use futures::lock::{Mutex, MutexGuard};
use futures::task::SpawnExt;
use log::{error, info};
use tokio::task::{AbortHandle, JoinSet};
use crate::sync::step::{Runner, Step};
use crate::tokio::step::{AsyncRunner, AsyncStep, StepResult};

pub fn log_step(message: String, is_error: bool) {
    if is_error {
        error!("{}", message);
    } else {
        info!("{}", message);
    }
}

pub async fn run_all_join_handles(join_set: Arc<Mutex<JoinSet<StepResult>>>) {
    let join_set = Arc::clone(&join_set);
    let mut join_set = join_set.lock().await;
    let mut is_panicked = false;
    while let Some(join_handle) = join_set.join_next().await {
        match join_handle {
            Ok(message) => {
                match message {
                    StepResult::Success(status) => {
                        log_step(status.status, false);
                    }
                    StepResult::Failure(status) => {
                        log_step(status.status, true);
                    }

                }
            }
            Err(join_error) => {
                if join_error.is_panic() {
                    is_panicked = true;
                    break;
                } else {
                    log_step(format!("Join error: {:?}", join_error), true);
                }
            }
        };
    }

    if is_panicked {
        join_set.abort_all();
    }
}

pub async fn mount_step_task<C: 'static + Sync + Send>(step: AsyncStep<C>, context: Arc<C>, throw_tolerant: bool, mut join_set: MutexGuard<'_, JoinSet<StepResult>>) -> AbortHandle {
    return join_set.spawn(async move {
        return match step.run(context).await {
            StepResult::Success(status) => {
                log_step(status.status.clone(), false);
                status
            }
            StepResult::Failure(status) => {
                if throw_tolerant {
                    panic!("{}", status.status);
                } else {
                    log_step(status.status.clone(), true);
                    status
                }
            }
        };
    });
}