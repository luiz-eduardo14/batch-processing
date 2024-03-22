use std::sync::Arc;

pub mod complex_step;
pub mod simple_step;
pub mod step_builder;

pub trait Runner where Self: Sized {
    fn run(self) -> Result<String, String>;
}

pub type StepCallback = Box<dyn FnOnce() + Send>;

type DeciderCallback = fn() -> bool;


pub struct Step {
    start_time: Option<u64>,
    end_time: Option<u64>,
    pub name: String,
    pub throw_tolerant: Option<bool>,
    decider: Option<DeciderCallback>,
    callback: Option<Box<dyn FnOnce()>>,
}

impl Runner for Step {
    fn run(self) -> Result<String, String> {
        return match self.callback {
            None => {
                if self.throw_tolerant.unwrap_or(false) {
                    return Err(format!("callback is required, please provide a callback to the step with name: {}", self.name));
                }
                panic!("callback is required, please provide a callback to the step with name: {}", self.name)
            }
            Some(callback) => {
                callback();
                Ok(format!("Step {} completed", self.name))
            }
        };
    }
}

unsafe impl Send for Step {}