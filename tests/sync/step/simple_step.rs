#[cfg(test)]
mod simple_step_test {
    use std::sync::{Arc, Mutex};
    use batch::sync::step::{Runner, simple_step};
    use batch::sync::step::simple_step::SimpleStepBuilderTrait;
    use batch::sync::step::step_builder::StepBuilderTrait;

    #[test]
    fn test_simple_step() {
        let test: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
        let test_clone = test.clone();
        let tasklet1 = move || {
            let mut test = test_clone.lock().unwrap();
            println!("Step 1: {:?}", test);
            test.push(1);
        };

        let step1 = simple_step::get("step".to_string())
            .throw_tolerant()
            .tasklet(Box::new(tasklet1)).build();

        let test_clone = test.clone();

        let tasklet2 = move || {
            let mut test = test_clone.lock().unwrap();
            println!("Step 2: {:?}", test);
            test.push(2);
        };

        let step2 = simple_step::get("step".to_string())
            .throw_tolerant()
            .tasklet(Box::new(tasklet2))
            .build();

        step1.run().expect("TODO: panic message");

        assert_eq!(test.lock().unwrap().len(), 1);

        step2.run().expect("TODO: panic message");

        assert_eq!(test.lock().unwrap().len(), 2);
    }
}