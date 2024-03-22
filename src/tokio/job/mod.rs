use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use futures::future::join_all;
use futures::lock::Mutex;
use futures::task::SpawnExt;
use log::{error, info};
use tokio::spawn;
use tokio::task::{JoinHandle, JoinSet};

use crate::sync::step::{Runner, Step};
use crate::tokio::step::{AsyncRunner, AsyncStep, StepResult};

pub mod job_builder;
mod utils;

pub struct AsyncJob <C: 'static> {
    pub name: String,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub steps: Vec<AsyncStep<C>>,
    pub multi_threaded: Option<bool>,
    pub max_tasks: Option<usize>,
}

#[derive(Debug, Clone)]
struct JobStatus {
    start_time: Option<u128>,
    end_time: Option<u128>,
    status: String,
    steps_status: Vec<StepResult>,
}

enum JobResult {
    Success(JobStatus),
    Failure(JobStatus),
}

fn generate_end_time() -> u128 {
    return SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
}

#[async_trait]
impl <C: 'static + Sync + Send> AsyncRunner<C, JobResult> for AsyncJob<C> {
    async fn run(mut self, context: Arc<C>) -> JobResult {
        let multi_threaded = self.multi_threaded.unwrap_or(false);
        let mut steps = self.steps;
        let steps_len = steps.len().clone();
        let start_time = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();

        if multi_threaded {
            info!("Running job {} with multi-threaded mode", self.name)
        } else {
            info!("Running job {} with single-threaded mode", self.name)
        }

        return if !multi_threaded {
            let mut steps_statuc_vec: Vec<StepResult> = Vec::new();
            for step in steps {
                let throw_tolerant = step.throw_tolerant.unwrap_or(false).clone();
                let context = Arc::clone(&context);
                let step_result = step.run(context).await;
                let step_result_clone: StepResult = step_result.clone();
                steps_statuc_vec.push(step_result);
                match step_result_clone {
                    StepResult::Failure(step_status) => {
                        if throw_tolerant {
                            return JobResult::Failure(JobStatus {
                                start_time,
                                end_time: generate_end_time(),
                                status: step_status.status,
                                steps_status: steps_statuc_vec,
                            });
                        } else {
                            error!("{}", step_status.status);
                        }
                    }
                    StepResult::Success(step_status) => utils::log_step(step_status),
                }
            }

            JobResult::Success(
                JobStatus {
                    start_time,
                    end_time: generate_end_time(),
                    status: format!("Job {} completed", self.name),
                    steps_status: steps_statuc_vec,
                }
            )
        } else {
            let join_set: Arc<Mutex<JoinSet<Result<String, String>>>> = Arc::new(Mutex::new(JoinSet::new()));
            let max_tasks = self.max_tasks.unwrap();

            if steps_len <= max_tasks {
                for step in steps {
                    let throw_tolerant = step.throw_tolerant.unwrap_or(false).clone();
                    let context = Arc::clone(&context);
                    let join_set = Arc::clone(&join_set);
                    let join_set = join_set.lock().await;
                    utils::mount_step_task(step, context, throw_tolerant, join_set).await;
                }

                let join_set = Arc::clone(&join_set);
                utils::run_all_join_handles(join_set).await;

                return Ok(format!("Job {} completed", self.name.clone()));
            }

            loop {
                let mut current_step = steps.pop();

                if current_step.is_none() {
                    break;
                }

                let step = current_step.unwrap();

                {
                    let context = Arc::clone(&context);
                    let throw_tolerant = step.throw_tolerant.unwrap_or(false).clone();
                    let join_set = Arc::clone(&join_set);
                    let join_set = join_set.lock().await;
                    utils::mount_step_task(step, context, throw_tolerant, join_set).await;
                }

                let mut is_full_tasks = false;

                {
                    let join_set = Arc::clone(&join_set);
                    let join_set = join_set.lock().await;
                    let join_set_len = join_set.len().clone();
                    is_full_tasks = join_set_len >= max_tasks;
                }

                if is_full_tasks {
                    let join_set = Arc::clone(&join_set);
                    utils::run_all_join_handles(join_set).await;
                }
            }

            Ok(format!("Job {} completed", self.name.clone()))
        }
    }
}

unsafe impl <C:'static> Send for AsyncJob<C> {}