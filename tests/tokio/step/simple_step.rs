#[cfg(test)]
mod async_simple_step_test {
    use std::future::Future;
    use std::sync::Arc;

    use tokio::spawn;

    use batch::tokio::step::{AsyncRunner, simple_step};
    use batch::tokio::step::simple_step::AsyncSimpleStepBuilderTrait;
    use batch::tokio::step::step_builder::AsyncStepBuilderTrait;

    struct ContextTest {
        pub name: String,
    }

    async fn run_callback(context: impl Future<Output = ContextTest> + Sized) -> Result<String, String> {
        let context = context.await;
        return Ok(format!("Hello, {}", context.name));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_simple_step() {

        let context = ContextTest {
            name: "test".to_string()
        };

        let string = String::from("test");

        let shared_context = Arc::new(context);

        let context = Arc::clone(&shared_context);

        let step = simple_step::get("test".to_string(), context).tasklet(Box::new(move |context| {
            return Box::pin(async move {
                return Ok(format!("Hello, {}", context.name));
            })
        })).build();

        let thread = spawn(async move {
            return step.run().await;
        });

        let result: Result<String, String> = thread.await.unwrap();

        assert_eq!(result, Ok("Hello, test".to_string()));
    }
}