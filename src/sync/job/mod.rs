use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

use log::{error, info};

use crate::core::job::{now_time, JobStatus};
use crate::core::step::StepStatus;
use crate::sync::step::{Decider, Runner, SyncStep};

pub mod job_builder;

/// Represents a synchronous job.
pub struct Job {
    /// The name of the job.
    pub name: String,
    /// The start time of the job execution.
    pub start_time: Option<u64>,
    /// The end time of the job execution.
    pub end_time: Option<u64>,
    /// The list of steps to be executed in the job.
    pub steps: Vec<SyncStep>,
    /// Indicates whether the job is to be executed in multithreaded mode.
    pub multi_threaded: Option<bool>,
    /// The maximum number of threads allowed for multithreaded execution.
    pub max_threads: Option<usize>,
}

impl Runner for Job {
    /// The output type of the job execution.
    type Output = JobStatus;

    /// Executes the synchronous job and returns its status.
    fn run(self) -> Self::Output {
        let start_time = now_time();
        let multi_threaded = self.multi_threaded.unwrap_or(false);
        let steps = self.steps;
        if multi_threaded {
            info!("Running job {} with multi-threaded mode", self.name)
        } else {
            info!("Running job {} with single-threaded mode", self.name)
        }

        fn log_step(result: Result<String, String>) {
            match result {
                Ok(success_message) => {
                    info!("{}", success_message);
                }
                Err(error_message) => {
                    error!("{}", error_message);
                }
            }
        }

        return if !multi_threaded {
            let mut steps_status_vec: Vec<StepStatus> = Vec::new();
            for step in steps {
                if step.is_run().clone() {
                    info!("Step {} is skipped", &step.name);
                    continue;
                }

                let throw_tolerant = step.throw_tolerant.unwrap_or(false).clone();
                let step_name = step.name.clone();

                info!("Running step {}", &step_name);

                let step_result = step.run();
                steps_status_vec.push(step_result.clone());

                match step_result.status {
                    Ok(success_message) => {
                        info!("{}", success_message);
                    }
                    Err(error_message) => {
                        if throw_tolerant {
                            error!("Error occurred in step {} but it is throw tolerant", &step_name);
                            panic!("{}", error_message);
                        } else {
                            error!("{}", error_message);
                        }
                    }
                }
            }

            let name = format!("Job {} completed", self.name);
            info!("{}", name);
            let end_time = now_time();
            JobStatus {
                status: Ok(name),
                end_time: Some(end_time),
                start_time: Some(start_time),
                steps_status: steps_status_vec,
            }
        } else {
            let max_threads = self.max_threads.unwrap_or(1);
            let threads: Arc<Mutex<Vec<JoinHandle<StepStatus>>>> = Arc::new(Mutex::new(Vec::new()));
            let mut steps_status_vec: Vec<StepStatus> = Vec::new();

            for step in steps {
                let threads = Arc::clone(&threads);
                {
                    let mut threads = threads.lock().unwrap();
                    threads.push(spawn(move || {
                        step.run()
                    }));
                }
                {
                    let mut threads = threads.lock().unwrap();
                    let threads_len = threads.len().clone();

                    if threads_len >= max_threads {
                        while let Some(join_handler) = threads.pop() {
                            let step_result = join_handler.join().unwrap();

                            log_step(step_result.status.clone());

                            steps_status_vec.push(step_result);
                        }
                    }
                }
            }

            let threads = Arc::clone(&threads);
            let mut threads = threads.lock().unwrap();

            if !threads.is_empty() {
                while let Some(join_handler) = threads.pop() {
                    let step_result = join_handler.join().unwrap();

                    log_step(step_result.status.clone());

                    steps_status_vec.push(step_result);
                }
            }

            JobStatus {
                status: Ok(format!("Job {} completed", self.name)),
                start_time: Some(start_time),
                end_time: Some(now_time()),
                steps_status: steps_status_vec,
            }
        };
    }
}

// Allows `Job` to be sent between threads safely.
unsafe impl Send for Job {}