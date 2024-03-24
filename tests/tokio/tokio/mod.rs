#[cfg(test)]
mod job_test {
    use batch::tokio::job::job_builder::{AsyncJobBuilder, AsyncJobBuilderTrait};
    use batch::tokio::job::JobResult;
    use batch::tokio::step::{AsyncRunner, AsyncStep};
    use batch::tokio::step::simple_step::{AsyncSimpleStepBuilder, AsyncSimpleStepBuilderTrait};
    use batch::tokio::step::step_builder::AsyncStepBuilderTrait;

    #[tokio::test(flavor = "multi_thread")]
    async fn job_simple_step() {
        fn generate_step(step_count: i8) -> AsyncStep {
            return AsyncSimpleStepBuilder::get(String::from(format!("step{}", step_count)))
                .tasklet(Box::new(move || {
                    return Box::pin(async move {
                        println!("{}", format!("Step {}", step_count));
                        return Ok(format!("Hello, {}", step_count));
                    });
                }))
                .build();
        }

        let mut job_builder = AsyncJobBuilder::get(String::from("sync-data"))
            .step(generate_step(1))
            .step(generate_step(2));

        for i in 3..=10 {
            job_builder = job_builder.step(generate_step(i));
        }

        let job = job_builder.build();
        match job.run().await {
            JobResult::Failure(job_result) => {
                println!("{:?}", job_result)
            }
            JobResult::Success(job_result) => {
                println!("{:?}", job_result)
            }
        }
    }
}