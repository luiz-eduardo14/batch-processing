use crate::step::Step;

pub trait StepBuilderTrait<I, O> {
    fn decider(self, decider: fn() -> bool) -> Self;
    fn throw_tolerant(self) -> Self;
    fn get(name: String) -> Self;
    fn validate(self) -> Self;
    fn build(self) -> Step;
}