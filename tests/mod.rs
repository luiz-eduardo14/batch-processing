#[cfg(test)]
mod simple_step_test {
    use std::thread;
    use std::thread::Thread;
    use batch::step::{Runner, simple_step, StepCallback};
    use batch::step::simple_step::SimpleStepBuilderTrait;
    use batch::step::step_builder::StepBuilderTrait;

    #[test]
    fn test_simple_step() {
        let mut test: Vec<i64> = Vec::new();

        let mut test: &mut Vec<i64> = &mut test;

        let tasklet1 = move || {
            println!("Step 1");

            test.push(1);
        };

        let step1 = simple_step::get("simple_step".to_string()).throw_tolerant(true).tasklet(Box::new(tasklet1)).build();
        
        step1.run().expect("TODO: panic message");

        // // RECOVER BORROWED OF test variable
        // let tasklet2 = move || {
        //     println!("Step 2");
        //     test.push(2);
        // };

        // let step2 = simple_step::get("simple_step".to_string()).throw_tolerant(true).tasklet(Box::new(tasklet2)).build();


    }
}