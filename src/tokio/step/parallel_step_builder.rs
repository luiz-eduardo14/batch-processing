/// A trait for building parallel asynchronous steps.
pub trait AsyncParallelStepBuilderTrait {
    /// sets the number of workers to use for parallel processing.
    /// Defaults to 1.
    /// # Parameters
    /// - `processor_concurrency_size`: The number of workers to use for parallel processing.
    /// # Returns `Self`
    /// The modified builder instance.
    fn processor_concurrency_size(self, processor_concurrency_size: usize) -> Self;
}