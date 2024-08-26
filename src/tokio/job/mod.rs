use std::sync::Arc;

use async_trait::async_trait;
use futures::lock::Mutex;
use log::{error, info};
use tokio::task::JoinSet;

use crate::core::job::{JobStatus, now_time};
use crate::core::step::StepStatus;
use crate::tokio::step::{AsyncStep, AsyncStepRunner, Decider};

pub mod job_builder;
mod utils;

/// A struct representing an asynchronous job.
pub struct AsyncJob {
    /// The name of the job.
    pub name: String,
    /// The start time of the job execution.
    pub start_time: Option<u64>,
    /// The end time of the job execution.
    pub end_time: Option<u64>,
    /// The list of steps to be executed in the job.
    pub steps: Vec<AsyncStep>,
    /// Indicates whether the job is to be executed in multithreaded mode.
    pub multi_threaded: Option<bool>,
    /// The maximum number of tasks allowed for multithreaded execution.
    pub max_tasks: Option<usize>,
}

#[async_trait]
impl AsyncStepRunner<JobStatus> for AsyncJob {
    /// Executes the asynchronous job and returns its result.
    async fn run(mut self) -> JobStatus {
        let multi_threaded = self.multi_threaded.unwrap_or(false);
        let mut steps = self.steps;
        let name = self.name.clone();
        let steps_len = steps.len().clone();
        let start_time = now_time();
        let mut steps_status_vec: Vec<StepStatus> = Vec::new();

        if multi_threaded {
            info!("Running job {} with multi-threaded mode", self.name)
        } else {
            info!("Running job {} with single-threaded mode", self.name)
        }

        return if !multi_threaded {
            for step in steps {
                if !step.decide().await {
                    info!("Skipping step {}", step.name);
                    continue;
                }

                let throw_tolerant = step.throw_tolerant.unwrap_or(false).clone();
                let step_result = step.run().await;
                let step_result_clone: StepStatus = step_result.clone();
                steps_status_vec.push(step_result);
                match step_result_clone.status {
                    Ok(message) => utils::log_step(Ok(message)),
                    Err(message) => {
                        if throw_tolerant {
                            return JobStatus {
                                name: self.name.clone(),
                                start_time: Some(start_time),
                                end_time: Some(now_time()),
                                status: Err(format!("Job {} failed", self.name)),
                                steps_status: steps_status_vec,
                            };
                        } else {
                            error!("{}", message);
                        }
                    }
                }
            }


            JobStatus {
                name,
                start_time: Some(start_time),
                end_time: Some(now_time()),
                status: Ok(format!("Job {} completed", self.name)),
                steps_status: steps_status_vec,
            }
        } else {
            let join_set: Arc<Mutex<JoinSet<StepStatus>>> = Arc::new(Mutex::new(JoinSet::new()));
            let max_tasks = self.max_tasks.unwrap();

            if steps_len <= max_tasks {
                for step in steps {
                    if !step.decide().await {
                        info!("Skipping step {}", step.name);
                        continue;
                    }
                    let throw_tolerant = step.throw_tolerant.unwrap_or(false).clone();
                    let join_set = Arc::clone(&join_set);
                    let join_set = join_set.lock().await;
                    utils::mount_step_task(step, throw_tolerant, join_set).await;
                }

                let join_set = Arc::clone(&join_set);
                let join_set_vec = utils::run_all_join_handles(join_set).await;

                return
                    JobStatus {
                        name: self.name.clone(),
                        start_time: Some(start_time),
                        end_time: Some(now_time()),
                        status: Ok(format!("Job {} completed", self.name)),
                        steps_status: join_set_vec,
                    };
            }

            loop {
                let current_step = steps.pop();

                if current_step.is_none() {
                    break;
                }

                let step = current_step.unwrap();

                {
                    let throw_tolerant = step.throw_tolerant.unwrap_or(false).clone();
                    let join_set = Arc::clone(&join_set);
                    let join_set = join_set.lock().await;
                    utils::mount_step_task(step, throw_tolerant, join_set).await;
                }

                let join_set = Arc::clone(&join_set);
                let join_set_len = join_set.lock().await.len().clone();
                let is_full_tasks = join_set_len >= max_tasks;

                if is_full_tasks {
                    let join_set = Arc::clone(&join_set);
                    let steps_status_vec_children = utils::run_all_join_handles(join_set).await;
                    steps_status_vec.extend(steps_status_vec_children);
                }
            }

            return
                JobStatus {
                    name: self.name.clone(),
                    start_time: Some(start_time),
                    end_time: Some(now_time()),
                    status: Ok(format!("Job {} completed", self.name)),
                    steps_status: steps_status_vec,
                };
        };
    }
}

// Allows `AsyncJob` to be sent between threads safely.
unsafe impl Send for AsyncJob {}