use crate::sync::step::{DeciderCallback, SyncStep};
use crate::sync::step::step_builder::StepBuilderTrait;

/// A trait for building complex synchronous steps.
pub trait ComplexStepBuilderTrait<I: Sized, O: Sized> {
    /// Sets the reader function for the step.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader function.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn reader(self, reader: Box<dyn Fn() -> Box<dyn Iterator<Item=I>> + Send>) -> Self;

    /// Sets the processor function for the step.
    ///
    /// # Arguments
    ///
    /// * `processor` - The processor function.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn processor(self, processor: Box<dyn Fn() -> Box<dyn Fn(I) -> O> + Send>) -> Self;

    /// Sets the writer function for the step.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer function.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn writer(self, writer: Box<dyn Fn() -> Box<dyn Fn(&Vec<O>) -> ()> + Send>) -> Self;

    /// Sets the chunk size for processing data in chunks.
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - The chunk size.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn chunk_size(self, chunk_size: usize) -> Self;
}

/// The default chunk size for processing data in chunks.
const DEFAULT_CHUNK_SIZE: usize = 1000;

impl<I: Sized + 'static, O: Sized + 'static> ComplexStepBuilderTrait<I, O> for ComplexStepBuilder<I, O> {
    fn reader(self, reader: Box<dyn Fn() -> Box<dyn Iterator<Item=I>> + Send>) -> Self {
        ComplexStepBuilder {
            reader: Some(reader),
            ..self
        }
    }

    fn processor(self, processor: Box<dyn Fn() -> Box<dyn Fn(I) -> O> + Send>) -> Self {
        ComplexStepBuilder {
            processor: Some(processor),
            ..self
        }
    }

    fn writer(self, writer: Box<dyn Fn() -> Box<dyn Fn(&Vec<O>) -> ()> + Send>) -> Self {
        ComplexStepBuilder {
            writer: Some(writer),
            ..self
        }
    }

    fn chunk_size(self, chunk_size: usize) -> Self {
        ComplexStepBuilder {
            chunk_size: Some(chunk_size),
            ..self
        }
    }
}

/// A builder struct for constructing complex synchronous steps.
pub struct ComplexStepBuilder<I: Sized, O: Sized> {
    /// The reader function for the step.
    reader: Option<Box<dyn Fn() -> Box<dyn Iterator<Item=I>> + Send>>,
    /// The processor function for the step.
    processor: Option<Box<dyn Fn() -> Box<dyn Fn(I) -> O> + Send>>,
    /// The writer function for the step.
    writer: Option<Box<dyn Fn() -> Box<dyn Fn(&Vec<O>) -> ()> + Send>>,
    /// The chunk size for processing data in chunks.
    chunk_size: Option<usize>,
    /// The synchronous step being constructed.
    step: SyncStep,
}

impl<I: Sized + 'static, O: Sized + 'static> StepBuilderTrait for ComplexStepBuilder<I, O> where Self: Sized {
    /// Sets the decider callback for the step.
    ///
    /// # Arguments
    ///
    /// * `decider` - The decider callback function.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn decider(self, decider: DeciderCallback) -> Self {
        ComplexStepBuilder {
            step: SyncStep {
                decider: Some(decider),
                ..self.step
            },
            ..self
        }
    }

    /// Configures the step to be tolerant to thrown exceptions.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance.
    fn throw_tolerant(self) -> Self {
        ComplexStepBuilder {
            step: SyncStep {
                throw_tolerant: Some(true),
                ..self.step
            },
            ..self
        }
    }

    /// Initializes a new builder instance with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the step.
    ///
    /// # Returns `Self`
    ///
    /// Returns a new builder instance.
    #[inline]
    fn get(name: String) -> Self {
        ComplexStepBuilder {
            reader: None,
            processor: None,
            writer: None,
            chunk_size: None,
            step: SyncStep {
                name,
                callback: None,
                decider: None,
                end_time: None,
                start_time: None,
                throw_tolerant: None,
            },
        }
    }

    /// Validates the builder configuration.
    ///
    /// # Returns `Self`
    ///
    /// Returns a modified builder instance if validation succeeds.
    fn validate(self) -> Self {
        if self.step.name.is_empty() {
            panic!("Name is required");
        }

        if self.reader.is_none() {
            panic!("Reader is required");
        }

        if self.processor.is_none() {
            panic!("Processor is required");
        }

        if self.writer.is_none() {
            panic!("Writer is required");
        }

        return self;
    }

    /// Builds and returns the configured synchronous step.
    ///
    /// # Returns `SyncStep`
    ///
    /// Returns the configured synchronous step.
    fn build(self) -> SyncStep {
        let mut current_self = self.validate();

        current_self.step.callback = Some(Box::new(move || {
            let reader = current_self.reader.unwrap();
            let processor = current_self.processor.unwrap().as_mut()();
            let writer = current_self.writer.unwrap().as_mut()();
            let chunk_size = current_self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
            let mut vec = Vec::with_capacity(chunk_size);

            for chunk in reader() {
                vec.push(processor(chunk));

                if vec.len() == chunk_size {
                    writer(&vec);
                    vec.clear();
                }
            }

            if !vec.is_empty() {
                writer(&vec);
            }
        }));

        return current_self.step;
    }
}

/// Initializes a new complex step builder with the given name.
///
/// # Arguments
///
/// * `name` - The name of the step.
///
/// # Returns `ComplexStepBuilder`
///
/// Returns a new complex step builder instance.
pub fn get<I: Sized + 'static, O: Sized + 'static>(name: String) -> ComplexStepBuilder<I, O> {
    ComplexStepBuilder::get(name)
}