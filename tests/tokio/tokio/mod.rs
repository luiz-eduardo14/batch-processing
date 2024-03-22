use std::future::Future;

struct Test {
    future: Box<dyn Future<Output=Result<String, String>>>,
}

#[cfg(test)]
mod job_test {
    use std::sync::Arc;
    use batch::tokio::job::job_builder::{AsyncJobBuilder, AsyncJobBuilderTrait};
    use batch::tokio::step::{AsyncRunner, AsyncStep};
    use batch::tokio::step::simple_step::{AsyncSimpleStepBuilder, AsyncSimpleStepBuilderTrait};
    use batch::tokio::step::step_builder::AsyncStepBuilderTrait;

    #[tokio::test(flavor = "multi_thread")]
    async fn job_simple_step() {
        let tasklet = move || {
            println!("Step 1");
        };

        let step1: AsyncStep<String> = AsyncSimpleStepBuilder::get(String::from("step1"))
            .tasklet(Box::new(move |context| {
                return Box::pin(async move {
                    println!("Step 1");
                    return Ok(format!("Hello, {}", context));
                });
            }))
            .build();

        let step2: AsyncStep<String> = AsyncSimpleStepBuilder::get(String::from("step2"))
            .throw_tolerant()
            .tasklet(Box::new(move |context| {
                return Box::pin(async move {
                    println!("Step 2");
                    return Ok(format!("Hello, {}", context));
                });
            }))
            .build();

        let job = AsyncJobBuilder::get(String::from("sync-data"))
            .step(step2)
            .step(step1)
            .multi_tasks(4)
            .build();

        let shared_context = Arc::new(String::from("test"));

        match job.run(shared_context).await {
            Ok(_) => {
                println!("Job finished");
            }
            Err(_) => {
                println!("Job failed");
            }
        }
    }
}