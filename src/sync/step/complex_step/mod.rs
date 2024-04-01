use crate::sync::step::{DeciderCallback, SyncStep};
use crate::sync::step::step_builder::StepBuilderTrait;

pub trait ComplexStepBuilderTrait<I: Sized, O: Sized> {
    fn reader(self, reader: Box<dyn Fn() -> Box<dyn Iterator<Item=I>>>) -> Self;
    fn processor(self, processor: Box<dyn Fn() -> Box<dyn Fn(I) -> O>>) -> Self;
    fn writer(self, writer: Box<dyn Fn() -> Box<dyn Fn(&Vec<O>) -> ()>>) -> Self;
    fn chunk_size(self, chunk_size: usize) -> Self;
}

const DEFAULT_CHUNK_SIZE: usize = 1000;

impl<I: Sized + 'static, O: Sized + 'static> ComplexStepBuilderTrait<I, O> for ComplexStepBuilder<I, O> {
    fn reader(self, reader: Box<dyn Fn() -> Box<dyn Iterator<Item=I>>>) -> Self {
        ComplexStepBuilder {
            reader: Some(reader),
            ..self
        }
    }

    fn processor(self, processor: Box<dyn Fn() -> Box<dyn Fn(I) -> O>>) -> Self {
        ComplexStepBuilder {
            processor: Some(processor),
            ..self
        }
    }

    fn writer(self, writer: Box<dyn Fn() -> Box<dyn Fn(&Vec<O>) -> ()>>) -> Self {
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

pub struct ComplexStepBuilder<I: Sized, O: Sized> {
    reader: Option<Box<dyn Fn() -> Box<dyn Iterator<Item=I>>>>,
    processor: Option<Box<dyn Fn() -> Box<dyn Fn(I) -> O>>>,
    writer: Option<Box<dyn Fn() -> Box<dyn Fn(&Vec<O>) -> ()>>>,
    chunk_size: Option<usize>,
    step: SyncStep,
}

impl<I: Sized + 'static, O: Sized + 'static> StepBuilderTrait for ComplexStepBuilder<I, O> where Self: Sized {
    fn decider(self, decider: DeciderCallback) -> Self {
        ComplexStepBuilder {
            step: SyncStep {
                decider: Some(decider),
                ..self.step
            },
            ..self
        }
    }

    fn throw_tolerant(self) -> Self {
        ComplexStepBuilder {
            step: SyncStep {
                throw_tolerant: Some(true),
                ..self.step
            },
            ..self
        }
    }

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

// impl <I: Sized + 'static, O: Sized + 'static> Runner for ComplexStepBuilder<I, O> {
//     fn run(self) {
//         let step = self.build();
//         step.run()
//     }
// }

pub fn get<I: Sized + 'static, O: Sized + 'static>(name: String) -> ComplexStepBuilder<I, O> {
    ComplexStepBuilder::get(name)
}