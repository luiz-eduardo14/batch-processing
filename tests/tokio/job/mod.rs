#[cfg(test)]
mod job_test {
    use futures::{Stream, stream};
    use batch::tokio::job::job_builder::{AsyncJobBuilder, AsyncJobBuilderTrait};
    use batch::tokio::job::JobResult;
    use batch::tokio::step::{AsyncRunner, AsyncStep};
    use batch::tokio::step::complex_step::{AsyncComplexStepBuilder, ComplexStepBuilderTrait};
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


    #[tokio::test]
    async fn job_complex_step() {
        fn generate_step(
            step_count: i32
        ) -> AsyncStep {
            let step_builder: AsyncComplexStepBuilder<i32, i32> = AsyncComplexStepBuilder::get("test".to_string())
                .reader(Box::new(move ||
                    {
                        let step_count = step_count;
                        return Box::pin(
                            async move {

                                let mut vec: Vec<i32>  = Vec::new();

                                for n in 0..= step_count {
                                    vec.push(n);
                                }

                                let stream: Box<dyn Stream<Item=i32> + Send + Unpin> = Box::new(stream::iter(vec));
                                stream
                            }
                        );
                    }))
                .processor(
                    Box::new(
                        move |item: i32| Box::pin(
                            async move {
                                item * 2
                            }
                        )
                    )
                )
                .writer(
                    Box::new(
                        move |items: Vec<i32>| Box::pin(
                            async move {
                                println!("{:?}", items);
                            }
                        )
                    )
                );

            return step_builder.build();
        }

        let mut job_builder = AsyncJobBuilder::get(String::from("job_complex_step"))
                .multi_tasks(4)
            ;

        for i in 1..=4 {
            job_builder = job_builder.step(generate_step(i));
        }

        let job = job_builder.build();

        job.run().await;
    }
}