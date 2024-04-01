pub mod complex_step;
pub mod simple_step;
pub mod step_builder;

pub trait Runner where Self: Sized {
    fn run(self) -> Result<String, String>;
}

pub trait Decider {
    fn is_run(&self) -> bool;
}

pub type StepCallback = Box<dyn FnOnce() + Send>;

pub type DeciderCallback = Box<dyn Fn() -> bool>;

pub struct SyncStep {
    #[allow(dead_code)]
    pub(crate) start_time: Option<u64>,
    #[allow(dead_code)]
    pub(crate) end_time: Option<u64>,
    pub name: String,
    pub throw_tolerant: Option<bool>,
    pub(crate) decider: Option<DeciderCallback>,
    pub(crate) callback: Option<Box<dyn FnOnce()>>,
}

impl Runner for SyncStep {
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

impl Decider for SyncStep {
    fn is_run (&self) -> bool {
        return match &self.decider {
            None => true,
            Some(decider) => decider(),
        };
    }
}

unsafe impl Send for SyncStep {}