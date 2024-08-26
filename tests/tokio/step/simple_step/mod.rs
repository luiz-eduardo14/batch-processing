#[cfg(all(feature = "async", test))]
mod async_simple_step_test {
    use std::sync::Arc;
    use batch_processing::tokio::step::{AsyncStepRunner, AsyncStep, simple_step};
    use batch_processing::tokio::step::simple_step::{AsyncSimpleStepBuilder, AsyncSimpleStepBuilderTrait};
    use batch_processing::tokio::step::step_builder::AsyncStepBuilderTrait;
    use tokio::join;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_simple_step() {
        let vector = Arc::new(Mutex::new(vec![1, 2, 3]));
        let shared_vector = Arc::clone(&vector);
        let step = AsyncSimpleStepBuilder::get("test".to_string())
            .tasklet(Box::new(
                move || {
                    let vector = shared_vector.clone();
                    return Box::pin(async move {
                        let mut vector = vector.lock().await;
                        vector.push(4);
                        println!("{}", format!("Step {}", String::new()));
                    });
                }
            )).build();

        step.run().await;

        let vector = Arc::clone(&vector);
        let shared_vector = vector.lock().await;

        println!("{:?}", shared_vector);
    }


    #[tokio::test]
    async fn test_simple_step_with_decider() {
        fn generate_step_with_sleep(name: String, millis_time: u64) -> AsyncStep {
            return simple_step::get(name.clone())
                .tasklet(Box::new(
                    move || {
                        let name = name.clone();
                        return Box::pin(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(millis_time)).await;
                            println!("{}", format!("Step {}", name));
                        });
                    }
                ))
                .decider(Box::new(
                    move || {
                        return Box::pin(async move {
                            return true;
                        });
                    }
                ))
                .build();
        }

        let step1 = generate_step_with_sleep("step1".to_string(), 100);
        let step2 = generate_step_with_sleep("step2".to_string(), 200);

        let (step1, step2) = join!(step1.run(), step2.run());

        assert!(step1.status.is_ok(), "The step 1 should be successful");
        assert!(step2.status.is_ok(), "The step 2 should be successful");
    }
}