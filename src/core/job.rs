use std::time::SystemTime;
use crate::core::step::StepStatus;

/// Represents the status of a job execution.
#[derive(Debug, Clone)]
pub struct JobStatus {
    /// The name of the job.
    pub name: String,
    /// The start time of the job execution.
    #[allow(dead_code)]
    pub start_time: Option<u128>,
    /// The end time of the job execution.
    #[allow(dead_code)]
    pub end_time: Option<u128>,
    /// The status message of the job execution.
    #[allow(dead_code)]
    pub status: Result<String, String>,
    /// The status of each step in the job execution.
    #[allow(dead_code)]
    pub steps_status: Vec<StepStatus>,
}

/// Generates the end time of a job execution.
pub fn now_time() -> u128 {
    return SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
}