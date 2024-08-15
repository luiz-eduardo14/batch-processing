use std::sync::Arc;

use async_trait::async_trait;
use futures::{SinkExt, Stream, StreamExt};
use futures::future::BoxFuture;
use tokio::join;
use tokio::sync::{mpsc, Mutex, MutexGuard};

use crate::tokio::step::{AsyncStep, DeciderCallback, DynAsyncCallback};
use crate::tokio::step::parallel_step_builder::AsyncParallelStepBuilderTrait;
use crate::tokio::step::step_builder::AsyncStepBuilderTrait;

/// Default chunk size used if not specified.
const DEFAULT_CHUNK_SIZE: usize = 1000;
const DEFAULT_PROCESSOR_CONCURRENCY_SIZE: usize = 1;

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
impl<I: Sized + 'static, O: Sized + 'static> ComplexStepBuilderTrait<I, O> for AsyncComplexStepBuilder<I, O> {
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
    /// The size of each processing task.
    /// Defaults to 1.
    processor_concurrency_size: usize,
    step: AsyncStep,
}

impl<I: Sized + Send + 'static + Unpin + Sync, O: Sized + Send + 'static + Sync> AsyncStepBuilderTrait for AsyncComplexStepBuilder<I, O>
where
    Self: Sized,
{
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
            processor_concurrency_size: DEFAULT_PROCESSOR_CONCURRENCY_SIZE,
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
            std::mem::take(&mut *shared_vec)
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
                let mut iterator = reader().await;


                let shared_is_finish = Arc::new(Mutex::new(false));
                let shared_vec: Arc<Mutex<Vec<O>>> = Arc::new(Mutex::new(Vec::new()));
                let shared_current_processor_concurrency_count = Arc::new(Mutex::new(0));

                let writer_reactor = Arc::clone(&writer);
                let (sender, mut receiver) = mpsc::channel::<Vec<O>>(32);
                let reactor = tokio::spawn(async move {
                    while let Some(vec) = receiver.recv().await {
                        writer_reactor(vec).await;
                    }
                });
                let shared_vec_iterator = Arc::clone(&shared_vec);
                let max_processor_concurrency_size = current_self.processor_concurrency_size;
                join!(
                    reactor,
                    async move {
                        loop {
                            let data = iterator.next().await;
                            if data.is_none() {
                                let mut is_finish = shared_is_finish.lock().await;
                                *is_finish = true;
                                break;
                            }
                            let data = data.unwrap();

                            let shared_vec = Arc::clone(&shared_vec_iterator);
                            let current_processor_concurrency_count = Arc::clone(&shared_current_processor_concurrency_count);

                            while {
                                let current_processor_concurrency_count = current_processor_concurrency_count.lock().await;
                                let count = *current_processor_concurrency_count;
                                count >= max_processor_concurrency_size
                            } {}

                            let processor = Arc::clone(&processor);
                            let sender = sender.clone();
                            {
                                let mut current_processor_concurrency_count = current_processor_concurrency_count.lock().await;
                                *current_processor_concurrency_count += 1
                            };
                            tokio::spawn(async move {
                                let sender = sender.clone();
                                let processor = Arc::clone(&processor);
                                let output = processor(data).await;
                                {
                                    let mut current_processor_concurrency_count = current_processor_concurrency_count.lock().await;
                                    *current_processor_concurrency_count -= 1;
                                }
                                let vec = Arc::clone(&shared_vec);
                                let mut vec = vec.lock().await;
                                vec.push(output);

                                if vec.len() >= chunk_size {
                                    let vec = transfer_mutex_vec_to_vec::<I, O>(vec);
                                    sender.send(vec).await.unwrap();
                                }
                            });
                        }
                    }
                );
                let vec = Arc::clone(&shared_vec);
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

impl<I: Sized + Send + 'static + Unpin + Sync, O: Sized + Send + 'static + Sync> AsyncParallelStepBuilderTrait for AsyncComplexStepBuilder<I, O>
where
    Self: Sized,
{
    /// Defaults to 1.
    /// # Parameters
    /// - `processor_concurrency_size`:  The number of workers to use for parallel processing.
    /// # Returns `Self`
    /// The modified builder instance.
    fn processor_concurrency_size(self, processor_concurrency_size: usize) -> Self {
        AsyncComplexStepBuilder {
            processor_concurrency_size,
            ..self
        }
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