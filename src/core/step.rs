use std::time::SystemTime;

/// Represents the status of a step execution.
#[derive(Debug, Clone)]
pub struct StepStatus {
    /// The start time of the step execution.
    pub start_time: Option<u128>,
    /// The end time of the step execution.
    pub end_time: Option<u128>,
    /// The status result of the step execution.
    pub status: Result<String, String>,
}

pub fn throw_tolerant_exception(throw_tolerant: bool, step_name: String) -> StepStatus {
    if throw_tolerant {
        return StepStatus {
            start_time: None,
            end_time: None,
            status: Ok(String::from("callback is required, please provide a callback to the step")),
        }
    }
    panic!("callback is required, please provide a callback to the step with name: {}", step_name)
}

pub fn mount_step_status(step_result: Result<String, String>, start_time: u128) -> StepStatus {
    let end_time = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    return match step_result {
        Ok(message) => StepStatus {
            start_time: Some(start_time),
            end_time: Some(end_time),
            status: Ok(message),
        },
        Err(message) => StepStatus {
            start_time: Some(start_time),
            end_time: None,
            status: Err(message),
        },
    };
}