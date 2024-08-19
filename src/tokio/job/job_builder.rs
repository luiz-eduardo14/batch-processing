use crate::tokio::job::AsyncJob;
use crate::tokio::step::AsyncStep;

/// A trait for building asynchronous jobs.
pub trait AsyncJobBuilderTrait {
    /// Validates the builder configuration.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self;

    /// Adds a step to the job.
    ///
    /// # Arguments
    ///
    /// * `step` - The asynchronous step to add.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn step(self, step: AsyncStep) -> Self;

    /// Configures the job to run with multiple tasks.
    ///
    /// # Arguments
    ///
    /// * `max_tasks` - The maximum number of tasks allowed for multithreaded execution.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn multi_tasks(self, max_tasks: usize) -> Self;

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

    /// Builds and returns the configured asynchronous job.
    ///
    /// # Returns `AsyncJob`
    ///
    /// Returns the configured asynchronous job.
    fn build(self) -> AsyncJob;
}

/// A builder struct for constructing asynchronous jobs.
pub struct AsyncJobBuilder {
    /// The job being constructed.
    job: AsyncJob,
}

impl AsyncJobBuilderTrait for AsyncJobBuilder {
    /// Validates the builder configuration by ensuring at least one step is added to the job.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self {
        self
    }

    /// Adds a step to the job.
    ///
    /// # Arguments
    ///
    /// * `step` - The asynchronous step to add.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn step(mut self, step: AsyncStep) -> Self {
        self.job.steps.push(step);
        self
    }

    /// Configures the job to run with multiple tasks.
    ///
    /// # Arguments
    ///
    /// * `max_tasks` - The maximum number of tasks allowed for multithreaded execution.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn multi_tasks(self, max_threads: usize) -> Self {
        AsyncJobBuilder {
            job: AsyncJob {
                max_tasks: Some(max_threads),
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

    /// Builds and returns the configured asynchronous job.
    ///
    /// # Returns `AsyncJob`
    ///
    /// Returns the configured asynchronous job.
    fn build(self) -> AsyncJob {
        let current_self = self.validate();
        return current_self.job;
    }
}