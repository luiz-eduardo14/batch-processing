use std::sync::Arc;
use async_trait::async_trait;
use futures::{StreamExt};
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use log::error;
use tokio::sync::{mpsc, Mutex};
use tokio::task::{JoinSet};
use crate::tokio::step::{AsyncStep, DeciderCallback};
use crate::tokio::step::parallel_step_builder::AsyncParallelStepBuilderTrait;
use crate::tokio::step::step_builder::AsyncStepBuilderTrait;

/// Default chunk size used if not specified.
const DEFAULT_CHUNK_SIZE: usize = 1000;
const DEFAULT_WORKERS_SIZE: usize = 1;

/// Alias for a callback function that processes input data asynchronously.
type DynParamAsyncCallback<I, O> = dyn Send + Sync + Fn(I) -> BoxFuture<'static, O>;
/// Alias for a callback function that reads input data asynchronously.
type ProcessorCallback<I, O> = Box<DynParamAsyncCallback<I, O>>;
/// Alias for a callback function that processes input data asynchronously and produces output.
type ReaderCallback<I> = Box<dyn Send + Sync + Fn() -> BoxFuture<'static, BoxStream<'static, I>>>;

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
    workers: usize,
    step: AsyncStep,
}

impl<I: Sized + Send + 'static, O: Sized + Send + 'static> AsyncStepBuilderTrait for AsyncComplexStepBuilder<I, O>
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
            workers: DEFAULT_WORKERS_SIZE,
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
        let throw_tolerant = current_self.step.throw_tolerant.unwrap_or(false);
        let step_name = Arc::new(current_self.step.name.clone());

        current_self.step.callback = Some(Box::new(move || {
            let reader = Box::pin(reader.clone());
            let processor = processor.clone();
            let writer = writer.clone();
            let chunk_size = current_self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
            let throw_tolerant = throw_tolerant.clone();
            let step_name = step_name.clone();
            return Box::pin(async move {
                let reader = Arc::clone(&reader);
                let processor = Arc::clone(&processor);
                let writer = Arc::clone(&writer);
                let throw_tolerant = throw_tolerant.clone();
                let step_name = Arc::clone(&step_name);

                let mut join_workers = JoinSet::new();
                let mut channels = Vec::new();
                let error = Arc::new(Mutex::new(false));
                for _ in 0..current_self.workers {
                    let (sender, receiver) = mpsc::channel::<I>(16);
                    let processor = Arc::clone(&processor);
                    let writer = Arc::clone(&writer);
                    let mut receiver = receiver;
                    let throw_tolerant = throw_tolerant.clone();
                    let error = Arc::clone(&error);
                    let step_name = Arc::clone(&step_name);
                    join_workers.spawn(async move {
                        let error = Arc::clone(&error);
                        let mut vec: Vec<O> = Vec::new();
                        let step_name = Arc::clone(&step_name);
                        while let Some(data) = receiver.recv().await {
                            let output = tokio::spawn(processor(data)).await;
                            if let Err(_) = output {
                                let mut error = error.lock().await;
                                *error = true;
                                if !throw_tolerant {
                                    panic!("step {}: Error to processing data", step_name);
                                } else {
                                    error!("step {}: Error to processing data", step_name);
                                    continue;
                                }
                            }
                            let output = output.unwrap();
                            vec.push(output);

                            if vec.len() >= chunk_size {
                                let vec_to_write = std::mem::take(&mut vec);
                                let writer_result = tokio::spawn(writer(vec_to_write)).await;
                                vec.clear();
                                if let Err(_) = writer_result {
                                    if !throw_tolerant {
                                        let mut error = error.lock().await;
                                        *error = true;
                                        panic!("step {}: Error to writing data", step_name);
                                    } else {
                                        error!("step {}: Error to writing data", step_name);
                                    }
                                }
                            }
                        }
                        if !vec.is_empty() {
                            let vec_to_write = std::mem::take(&mut vec);
                            let writer_result = tokio::spawn(writer(vec_to_write)).await;
                            if let Err(_) = writer_result {
                                if !throw_tolerant {
                                    let mut error = error.lock().await;
                                    *error = true;
                                    panic!("step {}: Error to writing data", step_name);
                                } else {
                                    error!("step {}: Error to writing data", step_name);
                                }
                            }
                        }
                    });
                    channels.push(sender);
                }
                let mut iterator = reader().await;
                let mut current_channel: usize = 0;
                while let Some(data) = iterator.next().await {
                    if !throw_tolerant {
                        let error = Arc::clone(&error);
                        let error = error.lock().await;
                        if *error {
                            join_workers.abort_all();
                            panic!("step {}: Error to processing data", step_name);
                        }
                    }
                    let sender = &mut channels[current_channel];
                    sender.send(data).await.unwrap();
                    if current_channel == current_self.workers - 1 {
                        current_channel = 0;
                    } else {
                        current_channel += 1;
                    }
                }
                drop(channels);
                while let Some(task_result) = join_workers.join_next().await {
                    if let Err(_) = task_result {
                        if !throw_tolerant {
                            let mut error = error.lock().await;
                            *error = true;
                            panic!("step {}: Error to processing data", step_name);
                        }
                        join_workers.abort_all();
                    }
                }
                let exist_error = error.lock().await;
                let exist_error = *exist_error;
                return exist_error;
            });
        }));

        return current_self.step;
    }
}

impl<I: Sized + Send + 'static + Sync, O: Sized + Send + 'static + Sync> AsyncParallelStepBuilderTrait for AsyncComplexStepBuilder<I, O>
where
    Self: Sized,
{
    /// Defaults to 1.
    /// # Parameters
    /// - `workers`:  The number of workers.
    /// # Returns `Self`
    /// The modified builder instance.
    fn workers(self, workers: usize) -> Self {
        AsyncComplexStepBuilder {
            workers,
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
pub fn get<I: Sized + 'static + Send, O: Sized + 'static + Send + Clone + Sync>(name: String) -> AsyncComplexStepBuilder<I, O> {
    AsyncComplexStepBuilder::get(name)
}