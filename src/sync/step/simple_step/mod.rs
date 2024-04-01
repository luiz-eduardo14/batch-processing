use crate::sync::step::{DeciderCallback, SyncStep, StepCallback};
use crate::sync::step::step_builder::StepBuilderTrait;

/// A trait for building simple synchronous steps.
pub trait SimpleStepBuilderTrait<I, O> {
    /// Configures the step with a tasklet callback.
    ///
    /// # Arguments
    ///
    /// * `step_callback` - The tasklet callback function.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance.
    fn tasklet(self, step_callback: StepCallback) -> Self;
}

/// A builder struct for constructing simple synchronous steps.
pub struct SimpleStepBuilder {
    /// The step being constructed.
    step: SyncStep,
}

impl StepBuilderTrait for SimpleStepBuilder {
    /// Sets the decider callback for the step.
    ///
    /// # Arguments
    ///
    /// * `decider` - The decider callback function.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance.
    fn decider(self, decider: DeciderCallback) -> Self {
        SimpleStepBuilder {
            step: SyncStep {
                decider: Some(decider),
                ..self.step
            }
        }
    }

    /// Configures the step to be tolerant to thrown exceptions.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance.
    fn throw_tolerant(self) -> Self {
        SimpleStepBuilder {
            step: SyncStep {
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
    /// # Returns
    ///
    /// Returns a new builder instance.
    #[inline]
    fn get(name: String) -> Self {
        SimpleStepBuilder {
            step: SyncStep {
                name,
                callback: None,
                decider: None,
                end_time: None,
                start_time: None,
                throw_tolerant: None,
            }
        }
    }

    /// Validates the builder configuration.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self {
        if self.step.callback.is_none() {
            panic!("Tasklet is required");
        }

        if self.step.name.is_empty() {
            panic!("Name is required");
        }

        return self;
    }

    /// Builds and returns the configured synchronous step.
    ///
    /// # Returns
    ///
    /// Returns the configured synchronous step.
    fn build(self) -> SyncStep {
        let current_self = self.validate();
        return current_self.step;
    }
}

impl SimpleStepBuilderTrait<fn(), fn()> for SimpleStepBuilder {
    /// Configures the step with a tasklet callback.
    ///
    /// # Arguments
    ///
    /// * `step_callback` - The tasklet callback function.
    ///
    /// # Returns
    ///
    /// Returns a modified builder instance.
    fn tasklet(self, step_callback: StepCallback) -> Self {
        return SimpleStepBuilder {
            step: SyncStep {
                callback: Some(step_callback),
                ..self.step
            }
        };
    }
}

/// Initializes a new simple step builder with the given name.
///
/// # Arguments
///
/// * `name` - The name of the step.
///
/// # Returns
///
/// Returns a new simple step builder instance.
pub fn get(name: String) -> SimpleStepBuilder {
    SimpleStepBuilder::get(name)
}