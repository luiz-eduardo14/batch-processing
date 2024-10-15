#[cfg(all(feature = "async", test))]
mod async_complex_step_test {
    use batch_processing::tokio::step::complex_step::{AsyncComplexStepBuilder, ComplexStepBuilderTrait};
    use batch_processing::tokio::step::step_builder::AsyncStepBuilderTrait;
    use batch_processing::tokio::step::AsyncStepRunner;
    use futures::{stream, Stream};
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_build() {
        let step_builder: AsyncComplexStepBuilder<String, String> = AsyncComplexStepBuilder::get("test".to_string())
            .reader(Box::new(move ||
            {
                return Box::pin(async move {
                    let stream: Pin<Box<dyn Stream<Item=String> + Send>> =
                        Box::pin(stream::iter(vec![String::new()]));
                    stream
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

    #[tokio::test]
    async fn test_fault_tolerant_is_false() {
        let vec = Arc::new(Mutex::new(Vec::new()));
        let vec_write = vec.clone();
        let step_builder: AsyncComplexStepBuilder<String, String> = AsyncComplexStepBuilder::get("test".to_string())
            .chunk_size(1)
            .reader(Box::new(move ||
                {
                    return Box::pin(async move {
                        let stream: Pin<Box<dyn Stream<Item=String> + Send>> =
                            Box::pin(stream::iter(vec!["test-failed".to_string(), "test".to_string()]));
                        stream
                    }
                    );
                }))
            .processor(
                Box::new(
                    move |item: String| Box::pin(
                        async move {
                            if item == "test-failed".to_string() {
                                panic!("test failed");
                            }
                            return item.to_uppercase();
                        }
                    )
                )
            )
            .writer(
                Box::new(
                    move |items: Vec<String>| {
                        let vec_write = vec_write.clone();
                        Box::pin(
                            async move {
                                let mut vec = vec_write.lock().await;
                                for item in items {
                                    vec.push(item);
                                }
                            }
                        )
                    }
                )
            );

        let step = step_builder.build();
        let step_result = step.run().await;
        let vec = vec.lock().await;
        assert_eq!(step_result.status.is_err(), true);
        assert_eq!(vec.len(), 0);
    }

    #[tokio::test]
    async fn test_fault_tolerant_is_true() {
        let vec = Arc::new(Mutex::new(Vec::new()));
        let vec_write = vec.clone();
        let step_builder: AsyncComplexStepBuilder<String, String> = AsyncComplexStepBuilder::get("test".to_string())
            .chunk_size(1)
            .throw_tolerant()
            .reader(Box::new(move ||
                {
                    return Box::pin(async move {
                        let stream: Pin<Box<dyn Stream<Item=String> + Send>> =
                            Box::pin(stream::iter(vec!["test-failed".to_string(), "test".to_string()]));
                        stream
                    }
                    );
                }))
            .processor(
                Box::new(
                    move |item: String| Box::pin(
                        async move {
                            if item == "test-failed".to_string() {
                                panic!("test failed");
                            }
                            return item.to_uppercase();
                        }
                    )
                )
            )
            .writer(
                Box::new(
                    move |items: Vec<String>| {
                        let vec_write = vec_write.clone();
                        Box::pin(
                            async move {
                                let mut vec = vec_write.lock().await;
                                for item in items {
                                    vec.push(item);
                                }
                            }
                        )
                    }
                )
            );

        let step = step_builder.build();
        let step_result = step.run().await;
        let vec = vec.lock().await;
        assert_eq!(step_result.status.is_err(), true);
        assert_eq!(vec.len(), 1);
    }
}