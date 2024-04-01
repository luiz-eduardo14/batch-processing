use std::sync::Arc;

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use futures::future::BoxFuture;
use tokio::sync::{Mutex, MutexGuard};

use crate::tokio::step::{AsyncStep, DeciderCallback, DynAsyncCallback};
use crate::tokio::step::step_builder::AsyncStepBuilderTrait;

/// Default chunk size used if not specified.
const DEFAULT_CHUNK_SIZE: usize = 1000;

/// Alias for a callback function that processes input data asynchronously.
type DynParamAsyncCallback<I, O> = dyn Send + Sync + Fn(I) -> BoxFuture<'static, O>;
/// Alias for a callback function that reads input data asynchronously.
type ProcessorCallback<I, O> = Box<DynParamAsyncCallback<I, O>>;
/// Alias for a callback function that processes input data asynchronously and produces output.
type ReaderCallback<I> = Box<DynAsyncCallback<Box<dyn Stream<Item=I> + Unpin + Send>>>;

#[async_trait]
pub trait ComplexStepBuilderTrait<I: Sized, O: Sized> {
    /// Sets the reader for the step.
    ///
    /// # Parameters
    ///
    /// - `reader`: A callback function for reading input data asynchronously.
    ///
    /// # Returns `Self`
    ///
    /// The modified builder instance.
    fn reader(self, reader: ReaderCallback<I>) -> Self;
    /// Sets the processor for the step.
    ///
    /// # Parameters
    ///
    /// - `processor`: A callback function for processing input data asynchronously.
    ///
    /// # Returns `Self`
    ///
    /// The modified builder instance.
    fn processor(self, processor: ProcessorCallback<I, O>) -> Self;
    /// Sets the writer for the step.
    ///
    /// # Parameters
    ///
    /// - `writer`: A callback function for writing output data asynchronously.
    ///
    /// # Returns `Self`
    ///
    /// The modified builder instance.
    fn writer(self, writer: Box<DynParamAsyncCallback<Vec<O>, ()>>) -> Self;
    /// Sets the chunk size for processing data in chunks.
    ///
    /// # Parameters
    ///
    /// - `chunk_size`: The size of each processing chunk, defaults to 1000.
    ///
    /// # Returns `Self`
    ///
    /// The modified builder instance.
    fn chunk_size(self, chunk_size: usize) -> Self;
}

/// Implementation of `ComplexStepBuilderTrait` for `AsyncComplexStepBuilder`.
#[async_trait]
impl<I: Sized + 'static + Send, O: Sized + 'static> ComplexStepBuilderTrait<I, O> for AsyncComplexStepBuilder<I, O> {
    fn reader(self, reader: ReaderCallback<I>) -> Self {
        AsyncComplexStepBuilder {
            reader: Some(reader),
            ..self
        }
    }

    fn processor(self, processor: ProcessorCallback<I, O>) -> Self {
        AsyncComplexStepBuilder {
            processor: Some(processor),
            ..self
        }
    }

    fn writer(self, writer: Box<DynParamAsyncCallback<Vec<O>, ()>>) -> Self {
        AsyncComplexStepBuilder {
            writer: Some(writer),
            ..self
        }
    }

    fn chunk_size(self, chunk_size: usize) -> Self {
        AsyncComplexStepBuilder {
            chunk_size: Some(chunk_size),
            ..self
        }
    }
}

/// An asynchronous complex step builder for processing data.
pub struct AsyncComplexStepBuilder<I: Sized, O: Sized> {
    reader: Option<ReaderCallback<I>>,
    processor: Option<ProcessorCallback<I, O>>,
    writer: Option<Box<DynParamAsyncCallback<Vec<O>, ()>>>,
    chunk_size: Option<usize>,
    step: AsyncStep,
}

impl<I: Sized + Send + 'static + Unpin + Sync, O: Sized + Send + 'static + Sync> AsyncStepBuilderTrait for AsyncComplexStepBuilder<I, O> where Self: Sized {
    /// Sets the decider for the step.
    ///
    /// # Parameters
    ///
    /// - `decider`: A callback function for deciding the flow of the step.
    ///
    /// # Returns `Self`
    ///
    /// The modified builder instance.
    fn decider(self, decider: DeciderCallback) -> Self {
        AsyncComplexStepBuilder {
            step: AsyncStep {
                decider: Some(decider),
                ..self.step
            },
            ..self
        }
    }

    /// Sets the step to be tolerant to exceptions.
    ///
    /// # Returns `Self`
    ///
    /// The modified builder instance.
    fn throw_tolerant(self) -> Self {
        AsyncComplexStepBuilder {
            step: AsyncStep {
                throw_tolerant: Some(true),
                ..self.step
            },
            ..self
        }
    }

    /// Retrieves a new step builder instance with a given name.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the step.
    ///
    /// # Returns `Self`
    ///
    /// A new step builder instance.
    #[inline]
    fn get(name: String) -> Self {
        AsyncComplexStepBuilder {
            reader: None,
            processor: None,
            writer: None,
            chunk_size: None,
            step: AsyncStep {
                name,
                callback: None,
                decider: None,
                throw_tolerant: None,
            },
        }
    }

    /// Validates the step builder instance.
    ///
    /// # Returns `Self`
    ///
    /// The validated builder instance.
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

    /// Builds and returns the asynchronous step.
    ///
    /// # Returns `AsyncStep`
    ///
    /// The built asynchronous step.
    fn build(self) -> AsyncStep {
        let mut current_self = self.validate();
        let reader = Arc::new(current_self.reader.unwrap());
        let processor = Arc::new(current_self.processor.unwrap());
        let writer = Arc::new(current_self.writer.unwrap());

        fn transfer_mutex_vec_to_vec<I: Sized + Send + 'static, O: Sized + Send + 'static>(mut shared_vec: MutexGuard<Vec<O>>) -> Vec<O> {
            let mut vec = vec![];

            while let Some(output) = shared_vec.pop() {
                vec.push(output);
            }

            vec
        }

        current_self.step.callback = Some(Box::new(move || {
            let reader = Box::pin(reader.clone());
            let processor = processor.clone();
            let writer = writer.clone();
            let chunk_size = current_self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
            return Box::pin(async move {
                let reader = Arc::clone(&reader);
                let processor = Arc::clone(&processor);
                let writer = Arc::clone(&writer);
                let shared_vec: Arc<Mutex<Vec<O>>> = Arc::new(Mutex::new(Vec::new()));
                let mut iterator = reader().await;

                while let Some(chunk) = iterator.next().await {
                    {
                        let processor = Arc::clone(&processor);
                        let output = processor(chunk).await;
                        let vec = Arc::clone(&shared_vec);
                        let mut vec = vec.lock().await;
                        vec.push(output);
                    }
                    {
                        let vec = Arc::clone(&shared_vec);
                        let mutex_vec = vec.lock().await;
                        if mutex_vec.len() == chunk_size {
                            let vec = transfer_mutex_vec_to_vec::<I, O>(mutex_vec);
                            writer(vec).await;
                        }
                    }
                }

                let vec = shared_vec.clone();
                let vec = vec.lock().await;

                if !vec.is_empty() {
                    let vec = transfer_mutex_vec_to_vec::<I, O>(vec);
                    writer(vec).await;
                }
            });
        }));

        return current_self.step;
    }
}

/// A function to retrieve an `AsyncComplexStepBuilder` instance with a given name.
///
/// # Parameters
///
/// - `name`: The name of the step.
///
/// # Returns `AsyncComplexStepBuilder`
///
/// An `AsyncComplexStepBuilder` instance.
pub fn get<I: Sized + 'static + Send + Unpin + Sync, O: Sized + 'static + Send + Clone + Sync>(name: String) -> AsyncComplexStepBuilder<I, O> {
    AsyncComplexStepBuilder::get(name)
}