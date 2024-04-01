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