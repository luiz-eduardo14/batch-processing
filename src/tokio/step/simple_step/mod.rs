use std::sync::Arc;
use crate::tokio::step::{AsyncStep, DeciderCallback, DynAsyncCallback};
use crate::tokio::step::step_builder::AsyncStepBuilderTrait;

/// This trait defines methods for building asynchronous steps with simple configurations.
pub trait AsyncSimpleStepBuilderTrait<I, O> {
    /// Sets the tasklet for the step.
    ///
    /// # Arguments
    ///
    /// * `step_callback` - The callback function for the step.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn tasklet(self, step_callback: Box<DynAsyncCallback<()>>) -> Self;
}

/// A builder struct for constructing asynchronous simple steps.
pub struct AsyncSimpleStepBuilder {
    callback: Option<Box<DynAsyncCallback<()>>>,
    step: AsyncStep,
}

impl AsyncStepBuilderTrait for AsyncSimpleStepBuilder {
    /// Sets the decider callback for the step.
    ///
    /// # Arguments
    ///
    /// * `decider` - The decider callback function.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn decider(self, decider: DeciderCallback) -> Self {
        AsyncSimpleStepBuilder {
            callback: self.callback,
            step: AsyncStep {
                decider: Some(decider),
                ..self.step
            }
        }
    }

    /// Sets the step to be tolerant to thrown errors.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn throw_tolerant(self) -> Self {
        AsyncSimpleStepBuilder {
            callback: self.callback,
            step: AsyncStep {
                throw_tolerant: Some(true),
                ..self.step
            }
        }
    }

    /// Initializes a new builder instance with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the step.
    ///
    /// # Returns `Self`
    ///
    /// Returns a new builder instance.
    #[inline]
    fn get(name: String) -> Self {
        AsyncSimpleStepBuilder {
            callback: None,
            step: AsyncStep {
                name,
                callback: None,
                decider: None,
                throw_tolerant: None,
            }
        }
    }

    /// Validates the builder configuration.
    ///
    /// # Panics
    ///
    /// Panics if tasklet or name is not provided.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self {
        if self.callback.is_none() {
            panic!("Tasklet is required");
        }

        if self.step.name.is_empty() {
            panic!("Name is required");
        }

        return self;
    }

    /// Builds and returns the configured asynchronous step.
    ///
    /// # Returns `AsyncStep`
    ///
    /// Returns the configured asynchronous step.
    fn build(self) -> AsyncStep {
        let current_self = self.validate();
        let mut step = current_self.step;
        let callback = Arc::new(current_self.callback.unwrap());
        step.callback = Some(Box::new(move || {
            let callback = Arc::clone(&callback);
            return Box::pin(async move {
                let callback = callback;
                callback().await;
                return Ok(());
            });
        }));
        return step;
    }
}

impl AsyncSimpleStepBuilderTrait<fn(), fn()> for AsyncSimpleStepBuilder {
    /// Sets the tasklet for the step.
    ///
    /// # Arguments
    ///
    /// * `step_callback` - The callback function for the step.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn tasklet(self, step_callback: Box<DynAsyncCallback<()>>) -> Self {
        return AsyncSimpleStepBuilder {
            callback: Some(step_callback),
            step: AsyncStep {
                ..self.step
            }
        };
    }
}

/// Returns a new `AsyncSimpleStepBuilder` instance with the given name.
///
/// # Arguments
///
/// * `name` - The name of the step.
///
/// # Returns `Self`
///
/// Returns a new `AsyncSimpleStepBuilder` instance.
pub fn get(name: String) -> AsyncSimpleStepBuilder {
    AsyncSimpleStepBuilder::get(name)
}
