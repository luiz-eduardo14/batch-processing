#[cfg(test)]
mod simple_step_test {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::thread::Thread;

    use batch::step::{Runner, simple_step};
    use batch::step::simple_step::SimpleStepBuilderTrait;
    use batch::step::step_builder::StepBuilderTrait;

    #[test]
    fn test_simple_step() {
        let test: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));

        let test_clone = test.clone();

        let thread1 = thread::spawn(move || {
            let vector = test_clone.clone();
            let mut test = vector.lock().unwrap();
            println!("Step 1: {:?}", test);
            test.push(1);
        });

        // let test_clone = test.clone();
        //
        // let thread2 = thread::spawn(move |test| || {
        //     let mut test = test_clone.lock().unwrap();
        //     println!("Step 2: {:?}", test);
        //     test.push(1);
        // });

        // match thread2.join() {
        //     Ok(_) => {
        //         // let test = test.lock().unwrap();
        //     }
        //     Err(_) => {}
        // }

        thread1.join().expect("TODO: panic message");

        println!("Test: {:?}", test.lock().unwrap());

        // let thread =thread::spawn(tasklet1);
        // let teread1 = thread::spawn(tasklet2);
        //
        // thread.join().expect("TODO: panic message");
        // teread1.join().expect("TODO: panic message");

        // let thread = thread::spawn(move || {
        let step1 = simple_step::get("simple_step".to_string()).throw_tolerant(true).tasklet(Box::new(move || {
            let mut test = test_clone.lock().unwrap();
            println!("Step 2: {:?}", test);
            test.push(1);
        })).build();



        // });
        //
        // thread.join().expect("TODO: panic message");

        //
        // step1.run().expect("TODO: panic message");

        // // RECOVER BORROWED OF test variable
        // let tasklet2 = move || {
        //     println!("Step 2");
        //     test.push(2);
        // };

        // let step2 = simple_step::get("simple_step".to_string()).throw_tolerant(true).tasklet(Box::new(tasklet2)).build();
    }
}