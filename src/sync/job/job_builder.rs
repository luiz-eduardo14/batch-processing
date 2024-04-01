use crate::sync::job::Job;
use crate::sync::step::SyncStep;

/// A trait for building synchronous jobs.
pub trait JobBuilderTrait {
    /// Validates the builder configuration.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self;

    /// Adds a step to the job.
    ///
    /// # Arguments
    ///
    /// * `step` - The synchronous step to add.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn step(self, step: SyncStep) -> Self;

    /// Configures the job to run in multithreaded mode.
    ///
    /// # Arguments
    ///
    /// * `max_threads` - The maximum number of threads allowed for multithreaded execution.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn multi_threaded(self, max_threads: usize) -> Self;

    /// Initializes a new builder instance with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the job.
    ///
    /// # Returns `Self`
    ///
    /// Returns a new builder instance.
    fn get(name: String) -> Self;

    /// Builds and returns the configured synchronous job.
    ///
    /// # Returns `Job`
    ///
    /// Returns the configured synchronous job.
    fn build(self) -> Job;
}

/// A builder struct for constructing synchronous jobs.
pub struct JobBuilder {
    /// The job being constructed.
    job: Job,
}

impl JobBuilderTrait for JobBuilder {
    /// Validates the builder configuration by ensuring at least one step is added to the job.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self {
        if self.job.steps.is_empty() {
            panic!("At least one step is required");
        }
        self
    }

    /// Adds a step to the job.
    ///
    /// # Arguments
    ///
    /// * `step` - The synchronous step to add.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn step(mut self, step: SyncStep) -> Self {
        self.job.steps.push(step);
        self
    }

    /// Configures the job to run in multithreaded mode.
    ///
    /// # Arguments
    ///
    /// * `max_threads` - The maximum number of threads allowed for multithreaded execution.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn multi_threaded(self, max_threads: usize) -> Self {
        JobBuilder {
            job: Job {
                max_threads: Some(max_threads),
                multi_threaded: Some(true),
                ..self.job
            }
        }
    }

    /// Initializes a new builder instance with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the job.
    ///
    /// # Returns `Self`
    ///
    /// Returns a new builder instance.
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

    /// Builds and returns the configured synchronous job.
    ///
    /// # Returns `Job`
    ///
    /// Returns the configured synchronous job.
    fn build(self) -> Job {
        let current_self = self.validate();
        return current_self.job;
    }
}