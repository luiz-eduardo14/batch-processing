# Batch Processing

A batch library for processing a list of items in parallel.

## Introduction

The idea is to process a large mass of data with low memory usage and high performance. The concept involves processing
a list of items in parallel, with a maximum number of items being processed at the same time.

## Example

This is a simple example of batch processing using asynchronous programming, processing a CSV file and inserting the data in database.

```rust,no_run
let step_builder: AsyncComplexStepBuilder<Result<StringRecord, csv_async::Error>, CarPrice> = AsyncComplexStepBuilder::get("csv_transfer".to_string())
    .reader(
        Box::new(move || {
            let csv_file = csv_path.clone();
            return Box::pin(async move {
                let csv_file = tokio::fs::File::open(csv_file).await.expect("Error opening file");
                let reader = csv_async::AsyncReader::from_reader(csv_file);
                let records = reader.into_records();
                let stream: Box<dyn Stream<Item=Result<StringRecord, csv_async::Error>> + Send + Unpin>
                    = Box::new(records);
                stream
            });
        })
    ).processor(
    Box::new(
        |csv_line: Result<StringRecord, csv_async::Error>| {
            let car_price = csv_line.unwrap();
            return Box::pin(
                async move {
                    let car_price = CarPrice {
                        id: None,
                        year: car_price.get(1).unwrap().parse::<i32>().ok(),
                        make: car_price.get(2).map(|s| Some(s.to_string())).unwrap_or(None),
                        model: car_price.get(3).map(|s| Some(s.to_string())).unwrap_or(None),
                        trim: car_price.get(4).map(|s| Some(s.to_string())).unwrap_or(None),
                        body: car_price.get(5).map(|s| Some(s.to_string())).unwrap_or(None),
                        transmission: car_price.get(6).map(|s| Some(s.to_string())).unwrap_or(None),
                        vin: car_price.get(7).map(|s| Some(s.to_string())).unwrap_or(None),
                        state: car_price.get(8).map(|s| Some(s.to_string())).unwrap_or(None),
                        condition: car_price.get(9).map(|s| s.parse::<i32>().ok()).unwrap_or(None),
                        odometer: car_price.get(10).map(|s| s.parse::<i32>().ok()).unwrap_or(None),
                        color: car_price.get(11).map(|s| Some(s.to_string())).unwrap_or(None),
                        interior: car_price.get(12).map(|s| Some(s.to_string())).unwrap_or(None),
                        seller: car_price.get(13).map(|s| Some(s.to_string())).unwrap_or(None),
                        nmr: car_price.get(14).map(|s| s.parse::<i32>().ok()).unwrap_or(None),
                        sellingprice: car_price.get(15).map(|s| s.parse::<i32>().ok()).unwrap_or(None),
                        saledate: None,
                    };
                    car_price
                }
            );
        }
    )
).writer(
    Box::new(
        move |vec_car_price: Vec<CarPrice>| {
            let pool = Arc::clone(&pool);
            let all_memory_usage = Arc::clone(&all_memory_usage);
            return Box::pin(
                async move {
                    let mut conn = pool.get().await.expect("Error getting connection");
                        insert_into(car_prices::table)
                            .values(vec_car_price)
                            .execute(&mut conn)
                            .await
                            .expect("Error inserting data");
                    let current_mem = PEAK_ALLOC.current_usage_as_mb() as i32;
                    all_memory_usage.lock().await.push(current_mem);
                }
            );
        }
    )
).chunk_size(2000);

let step = step_builder.build();

step.run().await;
```

## Steps for running integration tests

1. Install docker and docker-compose
2. Run the command `docker-compose up -d` to start the database
3. Run the command `cargo test -- --ignored` to run the integration tests
4. Run the command `docker-compose down` to stop the database
