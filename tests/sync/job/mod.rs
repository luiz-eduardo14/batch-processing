use std::future::Future;

struct Test {
    future: Box<dyn Future<Output=Result<String, String>>>
}

#[cfg(test)]
mod job_test {
    use batch::sync::job::job_builder::{JobBuilder, JobBuilderTrait};
    use batch::sync::step::Runner;
    use batch::sync::step::simple_step::{SimpleStepBuilder, SimpleStepBuilderTrait};
    use batch::sync::step::step_builder::StepBuilderTrait;

    #[test]
    fn job_simple_step() {
        let tasklet = move || {
            println!("Step 1");
        };
        let step1 = SimpleStepBuilder::get(String::from("step1")).tasklet(Box::new(tasklet)).build();
        let tasklet = move || {
            println!("Step 2");
        };
        let step2 = SimpleStepBuilder::get(String::from("step2")).throw_tolerant().tasklet(Box::new(tasklet)).build();

        let job = JobBuilder::get(String::from("sync-data"))
            // .step(step3)
            .step(step1)
            .step(step2)
            .multi_threaded(4)
            .build();

        match job.run() {
            Ok(_) => {
                println!("Job finished");
            },
            Err(_) => {
                println!("Job failed");
            }
        }
    }
}