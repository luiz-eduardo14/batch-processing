pub trait Processor<I, O> {
    fn process(&self, input: I) -> O;
}