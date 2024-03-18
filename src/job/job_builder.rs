use crate::job::Job;
use crate::step::Step;

pub trait JobBuilderTrait {
    fn validate(self) -> Self;
    fn step(self, step: Step) -> Self;
    fn multi_threaded(self, max_threads: usize) -> Self;
    #[inline]
    fn get(name: String) -> Self;
    fn build(self) -> Job;
}

pub struct JobBuilder {
    job: Job,
}

impl JobBuilderTrait for JobBuilder {
    fn validate(self) -> Self {
        if self.job.steps.is_empty() {
            panic!("At least one step is required");
        }
        self
    }

    fn step(mut self, step: Step) -> Self {
        self.job.steps.push(step);
        self
    }

    fn multi_threaded(self, max_threads: usize) -> Self {
        JobBuilder {
            job: Job {
                max_threads: Some(max_threads),
                multi_threaded: Some(true),
                ..self.job
            }
        }
    }

    #[inline]
    fn get(name: String) -> Self {
        JobBuilder {
            job: Job {
                name,
                start_time: None,
                end_time: None,
                steps: Vec::new(),
                multi_threaded: None,
                max_threads: None,
            }
        }
    }

    fn build(self) -> Job {
        let current_self = self.validate();
        return current_self.job;
    }
}