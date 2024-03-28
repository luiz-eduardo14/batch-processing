use std::sync::Arc;

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use futures::future::BoxFuture;
use tokio::sync::{Mutex, MutexGuard};

use crate::tokio::step::{AsyncStep, DeciderCallback, DynAsyncCallback};
use crate::tokio::step::step_builder::AsyncStepBuilderTrait;

type DynParamAsyncCallback<I, O> = dyn Send + Sync + Fn(I) -> BoxFuture<'static, O>;
type ProcessorCallback<I, O> = Box<DynParamAsyncCallback<I, O>>;
type ReaderCallback<I> = Box<DynAsyncCallback<Box<dyn Stream<Item=I> + Unpin + Send>>>;

#[async_trait]
pub trait ComplexStepBuilderTrait<I: Sized, O: Sized> {
    fn reader(self, reader: ReaderCallback<I>) -> Self;
    fn processor(self, processor: ProcessorCallback<I, O>) -> Self;
    fn writer(self, writer: Box<DynParamAsyncCallback<Vec<O>, ()>>) -> Self;
    fn chunk_size(self, chunk_size: usize) -> Self;
}

const DEFAULT_CHUNK_SIZE: usize = 1000;

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

pub struct AsyncComplexStepBuilder<I: Sized, O: Sized> {
    reader: Option<ReaderCallback<I>>,
    processor: Option<ProcessorCallback<I, O>>,
    writer: Option<Box<DynParamAsyncCallback<Vec<O>, ()>>>,
    chunk_size: Option<usize>,
    step: AsyncStep,
}

impl<I: Sized + Send + 'static + Unpin + Sync, O: Sized + Send + 'static + Sync> AsyncStepBuilderTrait for AsyncComplexStepBuilder<I, O> where Self: Sized {
    fn decider(self, decider: DeciderCallback) -> Self {
        AsyncComplexStepBuilder {
            step: AsyncStep {
                decider: Some(decider),
                ..self.step
            },
            ..self
        }
    }

    fn throw_tolerant(self) -> Self {
        AsyncComplexStepBuilder {
            step: AsyncStep {
                throw_tolerant: Some(true),
                ..self.step
            },
            ..self
        }
    }

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

                return Ok(String::from("Success"));
            });
        }));

        return current_self.step;
    }
}

// impl <I: Sized + 'static, O: Sized + 'static> Runner for ComplexStepBuilder<I, O> {
//     fn run(self) {
//         let step = self.build();
//         step.run()
//     }
// }

pub fn get<I: Sized + 'static + Send + Unpin + Sync, O: Sized + 'static + Send + Clone + Sync>(name: String) -> AsyncComplexStepBuilder<I, O> {
    AsyncComplexStepBuilder::get(name)
}