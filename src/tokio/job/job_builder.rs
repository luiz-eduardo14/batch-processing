use crate::tokio::job::AsyncJob;
use crate::tokio::step::AsyncStep;

pub trait AsyncJobBuilderTrait {
    fn validate(self) -> Self;
    fn step(self, step: AsyncStep) -> Self;
    fn multi_tasks(self, max_tasks: usize) -> Self;
    fn get(name: String) -> Self;
    fn build(self) -> AsyncJob;
}

pub struct AsyncJobBuilder {
    job: AsyncJob,
}

impl AsyncJobBuilderTrait for AsyncJobBuilder {
    fn validate(self) -> Self {
        if self.job.steps.is_empty() {
            panic!("At least one step is required");
        }
        self
    }

    fn step(mut self, step: AsyncStep) -> Self {
        self.job.steps.push(step);
        self
    }

    fn multi_tasks(self, max_threads: usize) -> Self {
        AsyncJobBuilder {
            job: AsyncJob {
                max_tasks: Some(max_threads),
                multi_threaded: Some(true),
                ..self.job
            }
        }
    }

    #[inline]
    fn get(name: String) -> Self {
        AsyncJobBuilder {
            job: AsyncJob {
                name,
                start_time: None,
                end_time: None,
                steps: Vec::new(),
                multi_threaded: None,
                max_tasks: None,
            }
        }
    }

    fn build(self) -> AsyncJob {
        let current_self = self.validate();
        return current_self.job;
    }
}