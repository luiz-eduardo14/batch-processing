use std::sync::Arc;
use crate::tokio::step::AsyncStep;

pub trait AsyncStepBuilderTrait<C> {
    fn decider(self, decider: fn() -> bool) -> Self;
    fn throw_tolerant(self) -> Self;
    fn get(name: String) -> Self;
    fn validate(self) -> Self;
    fn build(self) -> AsyncStep<C>;
}