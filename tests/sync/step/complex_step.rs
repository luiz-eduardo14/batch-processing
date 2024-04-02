#[cfg(test)]
mod complex_step_test {
    use std::sync::{Arc, Mutex};

    use batch_processing::sync::step::{complex_step, Runner};
    use batch_processing::sync::step::complex_step::ComplexStepBuilderTrait;
    use batch_processing::sync::step::step_builder::StepBuilderTrait;

    #[test]
    fn test_complex_step() {
        let test: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
        let test_clone = test.clone();

        let step = complex_step::get::<String, i64>("complex_step".to_string())
            .throw_tolerant()
            .reader(Box::new(|| {
                Box::new(vec![String::from("1")].into_iter())
            }))
            .processor(Box::new(|| {
                Box::new(|x: String| {
                    x.parse().unwrap()
                })
            }))
            .writer(Box::new(move || {
                let test = test_clone.clone();
                Box::new(
                    move |x: &Vec<i64>| {
                        test.lock().unwrap().push(x[0]);
                    }
                )
            }))
        .build();

        let step_result = step.run();

        let binding = test.clone();
        let test_clone = binding.lock().unwrap();

        assert_eq!(test_clone.len(), 1, "The length of the vector should be 1");

        assert_eq!(test_clone[0], 1, "The first element should be 1");

        assert!(step_result.status.is_ok(), "The step should be successful")
    }
}