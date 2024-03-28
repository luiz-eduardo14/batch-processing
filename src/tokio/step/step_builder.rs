use crate::tokio::step::{AsyncStep, DeciderCallback};

pub trait AsyncStepBuilderTrait {
    fn decider(self, decider: DeciderCallback) -> Self;
    fn throw_tolerant(self) -> Self;
    fn get(name: String) -> Self;
    fn validate(self) -> Self;
    fn build(self) -> AsyncStep;
}