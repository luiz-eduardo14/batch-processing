use crate::sync::step::{DeciderCallback, SyncStep};

/// A trait for building synchronous steps.
pub trait StepBuilderTrait {
    /// Sets the decider callback for the step.
    ///
    /// # Arguments
    ///
    /// * `decider` - The decider callback function.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance.
    fn decider(self, decider: DeciderCallback) -> Self;

    /// Configures the step to be tolerant to thrown exceptions.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance.
    fn throw_tolerant(self) -> Self;

    /// Initializes a new builder instance with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the step.
    ///
    /// # Returns
    ///
    /// Returns a new builder instance.
    fn get(name: String) -> Self;

    /// Validates the builder configuration.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self;

    /// Builds and returns the configured synchronous step.
    ///
    /// # Returns
    ///
    /// Returns the configured synchronous step.
    fn build(self) -> SyncStep;
}