use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

use log::{error, info};

use crate::sync::step::{Runner, Step};

pub mod job_builder;

pub struct Job {
    pub name: String,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub steps: Vec<Step>,
    pub multi_threaded: Option<bool>,
    pub max_threads: Option<usize>,
}

impl Runner for Job {
    fn run(self) -> Result<String, String> {
        let multi_threaded = self.multi_threaded.unwrap_or(false);

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
            for step in self.steps {
                let throw_tolerant = step.throw_tolerant.unwrap_or(false).clone();

                match step.run() {
                    Ok(success_message) => {
                        info!("{}", success_message);
                    }
                    Err(error_message) => {
                        if throw_tolerant {
                            panic!("{}", error_message);
                        } else {
                            error!("{}", error_message);
                        }
                    }
                }
            }

            let name = format!("Job {} completed", self.name);
            info!("{}", name);
            Ok(name)
        } else {
            let max_threads = self.max_threads.unwrap_or(1);
            let threads: Arc<Mutex<Vec<JoinHandle<Result<String, String>>>>> = Arc::new(Mutex::new(Vec::new()));
            let steps: Vec<Step> = self.steps;
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
                        while let Some(joinHandler) = threads.pop() {
                            let result = joinHandler.join().unwrap();
                            log_step(result);
                        }
                    }
                }
            }

            let threads = Arc::clone(&threads);
            let mut threads = threads.lock().unwrap();

            if !threads.is_empty() {
                while let Some(joinHandler) = threads.pop() {
                    let result = joinHandler.join().unwrap();
                    log_step(result);
                }
            }

            Ok(format!("Job {} completed", self.name))
        };
    }
}

unsafe impl Send for Job {}