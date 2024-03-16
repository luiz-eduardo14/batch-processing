use std::rc::Rc;
use std::sync::Arc;

pub mod complex;
pub mod simple_step;
pub mod step_builder;

pub trait Runner<T> where Self: Sized {
    fn run(self) -> Result<(), String> where T: Default;
    fn run_with_context(self, value: T) -> Result<(), String>;
}

pub type StepCallback<T> = Box<dyn FnOnce(T)>;

type DeciderCallback = fn() -> bool;

pub struct Step<T> {
    start_time: Option<u64>,
    end_time: Option<u64>,
    name: String,
    throw_tolerant: Option<bool>,
    decider: Option<DeciderCallback>,
    callback: Option<StepCallback<T>>,
    context: Option<Arc<T>>,
}

impl<T> Runner<T> for Step<T> {
    fn run(self) -> Result<(), String> where T: Default {
        return match self.callback {
            None => {
                if self.throw_tolerant.unwrap_or(false) {
                    return Err(format!("callback is required, please provide a callback to the step with name: {}", self.name));
                }
                panic!("callback is required, please provide a callback to the step with name: {}", self.name)
            }
            Some(callback) => {
                Ok(callback(Default::default()))
            }
        };
    }

    fn run_with_context(self, value: T) -> Result<(), String> {
        return match self.callback {
            None => {
                if self.throw_tolerant.unwrap_or(false) {
                    return Err(format!("callback is required, please provide a callback to the step with name: {}", self.name));
                }
                panic!("callback is required, please provide a callback to the step with name: {}", self.name)
            }
            Some(callback) => {
                Ok(callback(value))
            }
        };
    }
}