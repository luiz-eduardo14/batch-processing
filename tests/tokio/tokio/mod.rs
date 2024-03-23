use std::future::Future;

struct Test {
    future: Box<dyn Future<Output=Result<String, String>>>,
}

#[cfg(test)]
mod job_test {
    use std::sync::Arc;

    use batch::tokio::job::job_builder::{AsyncJobBuilder, AsyncJobBuilderTrait};
    use batch::tokio::job::JobResult;
    use batch::tokio::step::{AsyncRunner, AsyncStep};
    use batch::tokio::step::simple_step::{AsyncSimpleStepBuilder, AsyncSimpleStepBuilderTrait};
    use batch::tokio::step::step_builder::AsyncStepBuilderTrait;

    #[tokio::test(flavor = "multi_thread")]
    async fn job_simple_step() {
        let tasklet = move || {
            println!("Step 1");
        };

        fn generate_step(step_count: i8) -> AsyncStep<String> {
            return AsyncSimpleStepBuilder::get(String::from(format!("step{}", step_count)))
                .tasklet(Box::new(move |context| {
                    return Box::pin(async move {
                        println!("{}", format!("Step {}", step_count));
                        return Ok(format!("Hello, {}", context));
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

        let shared_context = Arc::new(String::from("test"));

        match job.run(shared_context).await {
            JobResult::Failure(job_result) => {
                println!("Job failed");
            }
            JobResult::Success(job_result) => {
                println!("Job finished");
            }
        }
    }
}