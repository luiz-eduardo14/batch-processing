use crate::tokio::step::{AsyncStep, DeciderCallback, DynAsyncCallback};
use crate::tokio::step::step_builder::AsyncStepBuilderTrait;

pub trait AsyncSimpleStepBuilderTrait<I, O> {
    fn tasklet(self, step_callback: Box<DynAsyncCallback<Result<String, String>>>) -> Self;
}

pub struct AsyncSimpleStepBuilder {
    step: AsyncStep,
}

impl AsyncStepBuilderTrait for AsyncSimpleStepBuilder {
    fn decider(self, decider: DeciderCallback) -> Self {
        AsyncSimpleStepBuilder {
            step: AsyncStep {
                decider: Some(decider),
                ..self.step
            }
        }
    }

    fn throw_tolerant(self) -> Self {
        AsyncSimpleStepBuilder {
            step: AsyncStep {
                throw_tolerant: Some(true),
                ..self.step
            }
        }
    }

    #[inline]
    fn get(name: String) -> Self {
        AsyncSimpleStepBuilder {
            step: AsyncStep {
                name,
                callback: None,
                decider: None,
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

    fn build(self) -> AsyncStep {
        let current_self = self.validate();
        return current_self.step;
    }
}

impl AsyncSimpleStepBuilderTrait<fn(), fn()> for AsyncSimpleStepBuilder {
    fn tasklet(self, step_callback: Box<DynAsyncCallback<Result<String, String>>>) -> Self {
        return AsyncSimpleStepBuilder {
            step: AsyncStep {
                callback: Some(step_callback),
                ..self.step
            }
        };
    }
}

pub fn get(name: String) -> AsyncSimpleStepBuilder {
    AsyncSimpleStepBuilder::get(name)
}