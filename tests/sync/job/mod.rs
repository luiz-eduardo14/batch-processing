#[cfg(test)]
mod job_test {
    use batch_processing::sync::job::job_builder::{JobBuilder, JobBuilderTrait};
    use batch_processing::sync::step::Runner;
    use batch_processing::sync::step::simple_step::{SimpleStepBuilder, SimpleStepBuilderTrait};
    use batch_processing::sync::step::step_builder::StepBuilderTrait;

    use crate::utils::log::enable_test_log;

    #[test]
    fn job_simple_step() {
        enable_test_log();
        let tasklet = move || {
            println!("Step 1");
        };
        let step1 = SimpleStepBuilder::get(String::from("step1"))
            .tasklet(Box::new(tasklet))
            .decider(Box::new(|| false))
        .build();
        let tasklet = move || {
            println!("Step 2");
        };
        let step2 = SimpleStepBuilder::get(String::from("step2")).throw_tolerant().tasklet(Box::new(tasklet)).build();

        let job = JobBuilder::get(String::from("sync-data"))
            .step(step1)
            .step(step2)
            .multi_threaded(4)
            .build();

        match job.run().status {
            Ok(_) => {
                println!("Job finished");
            },
            Err(_) => {
                println!("Job failed");
            }
        }
    }
}