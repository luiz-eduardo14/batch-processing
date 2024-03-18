#[cfg(test)]
mod job_test {
    use std::thread;
    use std::thread::Thread;
    use std::time::Duration;
    use batch::job::job_builder::{JobBuilder, JobBuilderTrait};
    use batch::step::Runner;
    use batch::step::simple_step::{SimpleStepBuilder, SimpleStepBuilderTrait};
    use batch::step::step_builder::StepBuilderTrait;

    #[test]
    fn job_simple_step() {
        let tasklet = move || {
            println!("Step 1")
        };
        let step1 = SimpleStepBuilder::get(String::from("step1")).throw_tolerant().tasklet(Box::new(tasklet)).build();
        let tasklet = move || {
            println!("Step 2")
        };
        let step2 = SimpleStepBuilder::get(String::from("step2")).throw_tolerant().tasklet(Box::new(tasklet)).build();

        let tasklet = move || {
            thread::sleep(Duration::from_secs(1));
            println!("Step 3")
        };
        let step3= SimpleStepBuilder::get(String::from("step2")).throw_tolerant().tasklet(Box::new(tasklet)).build();

        let job = JobBuilder::get(String::from("job1"))
            .step(step3)
            .step(step1)
            .step(step2)
            .multi_threaded(4)
            .build();

        job.run().expect("TODO: panic message");
    }
}