use crate::sync::step::{DeciderCallback, SyncStep, StepCallback};
use crate::sync::step::step_builder::StepBuilderTrait;

pub trait SimpleStepBuilderTrait<I, O> {
    fn tasklet(self, step_callback: StepCallback) -> Self;
}

pub struct SimpleStepBuilder {
    step: SyncStep,
}

impl StepBuilderTrait for SimpleStepBuilder {
    fn decider(self, decider: DeciderCallback) -> Self {
        SimpleStepBuilder {
            step: SyncStep {
                decider: Some(decider),
                ..self.step
            }
        }
    }

    fn throw_tolerant(self) -> Self {
        SimpleStepBuilder {
            step: SyncStep {
                throw_tolerant: Some(true),
                ..self.step
            }
        }
    }

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

    fn validate(self) -> Self {
        if self.step.callback.is_none() {
            panic!("Tasklet is required");
        }

        if self.step.name.is_empty() {
            panic!("Name is required");
        }

        return self;
    }

    fn build(self) -> SyncStep {
        let current_self = self.validate();
        return current_self.step;
    }
}

impl SimpleStepBuilderTrait<fn(), fn()> for SimpleStepBuilder {
    fn tasklet(self, step_callback: StepCallback) -> Self {
        return SimpleStepBuilder {
            step: SyncStep {
                callback: Some(step_callback),
                ..self.step
            }
        };
    }
}

pub fn get(name: String) -> SimpleStepBuilder {
    SimpleStepBuilder::get(name)
}