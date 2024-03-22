use std::sync::Arc;
use crate::tokio::step::{AsyncStep, DynAsyncCallback};
use crate::tokio::step::step_builder::AsyncStepBuilderTrait;

pub trait AsyncSimpleStepBuilderTrait<I, O, C> {
    fn tasklet(self, step_callback: Box<DynAsyncCallback<Arc<C>, Result<String, String>>>) -> Self;
}

pub struct AsyncSimpleStepBuilder<C: 'static> {
    step: AsyncStep<C>,
}

impl <C> AsyncStepBuilderTrait<C> for AsyncSimpleStepBuilder<C> {
    fn decider(self, decider: fn() -> bool) -> Self {
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

    fn build(self) -> AsyncStep<C> {
        let current_self = self.validate();
        return current_self.step;
    }
}

impl <C> AsyncSimpleStepBuilderTrait<fn(), fn(), C> for AsyncSimpleStepBuilder<C> {
    fn tasklet(self, step_callback: Box<DynAsyncCallback<Arc<C>, Result<String, String>>>) -> Self {
        return AsyncSimpleStepBuilder {
            step: AsyncStep {
                callback: Some(step_callback),
                ..self.step
            }
        };
    }
}

pub fn get<C>(name: String) -> AsyncSimpleStepBuilder<C> {
    AsyncSimpleStepBuilder::get(name)
}