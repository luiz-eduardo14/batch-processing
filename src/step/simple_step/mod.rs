use crate::step::{Step, StepCallback};
use crate::step::step_builder::StepBuilderTrait;

pub trait SimpleStepBuilderTrait<I, O> {
    fn tasklet(self, step_callback: StepCallback) -> Self;
}

pub struct SimpleStepBuilder {
    step: Step,
}

impl StepBuilderTrait<fn(), fn()> for SimpleStepBuilder {
    // fn chunk(self, chunk_size: u32) -> Self {
    //     SimpleStepBuilder {
    //         step: Step {
    //             chunk_size: Some(chunk_size),
    //             ..self.step
    //         }
    //     }
    // }

    fn decider(self, decider: fn() -> bool) -> Self {
        SimpleStepBuilder {
            step: Step {
                decider: Some(decider),
                ..self.step
            }
        }
    }

    fn throw_tolerant(self) -> Self {
        SimpleStepBuilder {
            step: Step {
                throw_tolerant: Some(true),
                ..self.step
            }
        }
    }

    #[inline]
    fn get(name: String) -> Self {
        SimpleStepBuilder {
            step: Step {
                name,
                callback: None,
                // chunk_size: None,
                decider: None,
                end_time: None,
                start_time: None,
                throw_tolerant: None,
            }
        }
    }

    fn validate(self) -> Self {
        // if self.step.chunk_size.is_none() {
        //     panic!("chunk_size is required");
        // }

        if self.step.callback.is_none() {
            panic!("Tasklet is required");
        }

        if self.step.name.is_empty() {
            panic!("Name is required");
        }

        if self.step.throw_tolerant.is_none() {
            panic!("throw_tolerant is required");
        }

        return self;
    }

    fn build(self) -> Step {
        let current_self = self.validate();
        return current_self.step;
    }
}

impl SimpleStepBuilderTrait<fn(), fn()> for SimpleStepBuilder {
    fn tasklet(self, step_callback: StepCallback) -> Self {
        return SimpleStepBuilder {
            step: Step {
                callback: Some(step_callback),
                ..self.step
            }
        };
    }
}

pub fn get(name: String) -> SimpleStepBuilder {
    SimpleStepBuilder::get(name)
}