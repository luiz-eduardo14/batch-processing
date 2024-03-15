use std::rc::Rc;

pub mod complex;
pub mod simple_step;
pub mod step_builder;

pub trait Runner {
    fn run(self) -> Result<(), String>;
}

pub type StepCallback = Box<dyn FnOnce()>;

type DeciderCallback = fn() -> bool;

pub struct Step {
    start_time: Option<u64>,
    end_time: Option<u64>,
    name: String,
    throw_tolerant: Option<bool>,
    decider: Option<DeciderCallback>,
    callback: Option<StepCallback>,
}

impl Runner for Step {
    fn run(self) -> Result<(), String> {
        return match self.callback {
            None => {
                if self.throw_tolerant.unwrap_or(false) {
                    return Err(format!("callback is required, please provide a callback to the step with name: {}", self.name))
                }
                panic!("callback is required, please provide a callback to the step with name: {}", self.name)
            }
            Some(callback) => {
                Ok(callback())
            }
        };
    }
}