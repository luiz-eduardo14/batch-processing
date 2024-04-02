#[cfg(all(feature = "async", test))]
mod async_complex_step_test {
    use futures::{stream, Stream};

    use batch_processing::tokio::step::AsyncStepRunner;
    use batch_processing::tokio::step::complex_step::{AsyncComplexStepBuilder, ComplexStepBuilderTrait};
    use batch_processing::tokio::step::step_builder::AsyncStepBuilderTrait;

    #[tokio::test]
    async fn test_build() {
        let step_builder: AsyncComplexStepBuilder<String, String> = AsyncComplexStepBuilder::get("test".to_string())
            .reader(Box::new(move ||
                {
                    return Box::pin(
                        async move {
                            let stream: Box<dyn Stream<Item=String> + Send + Unpin> = Box::new(stream::iter(vec![String::new()]));
                            return stream;
                        }
                    );
                }))
            .processor(
                Box::new(
                    move |item: String| Box::pin(
                        async move {
                            item.to_uppercase()
                        }
                    )
                )
            )
            .writer(
                Box::new(
                    move |items: Vec<String>| Box::pin(
                        async move {
                            println!("{:?}", items);
                        }
                    )
                )
            );

        let step = step_builder.build();

        step.run().await;
    }
}