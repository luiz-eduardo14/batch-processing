use crate::sync::step::Step;

pub trait StepBuilderTrait {
    fn decider(self, decider: fn() -> bool) -> Self;
    fn throw_tolerant(self) -> Self;
    fn get(name: String) -> Self;
    fn validate(self) -> Self;
    fn build(self) -> Step;
}