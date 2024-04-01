use crate::tokio::step::{AsyncStep, DeciderCallback};

/// A trait for building asynchronous steps.
pub trait AsyncStepBuilderTrait {
    /// Sets the decider callback for the step.
    ///
    /// # Arguments
    ///
    /// * `decider` - The decider callback function.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn decider(self, decider: DeciderCallback) -> Self;

    /// Sets the step to be tolerant to thrown errors.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn throw_tolerant(self) -> Self;

    /// Initializes a new builder instance with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the step.
    ///
    /// # Returns `Self`
    ///
    /// Returns a new builder instance.
    fn get(name: String) -> Self;

    /// Validates the builder configuration.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self;

    /// Builds and returns the configured asynchronous step.
    ///
    /// # Returns `AsyncStep`
    ///
    /// Returns the configured asynchronous step.
    fn build(self) -> AsyncStep;
}