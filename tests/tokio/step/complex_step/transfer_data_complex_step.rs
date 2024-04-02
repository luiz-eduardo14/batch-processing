#[cfg(test)]
mod async_transfer_data_complex_step_test {
    use std::fs::File;
    use std::io;
    use std::path::Path;
    use std::sync::Arc;

    use chrono::NaiveDate;
    use csv_async::StringRecord;
    use deadpool_postgres::Timeouts;
    use diesel::{insert_into, Insertable, Queryable};
    use diesel_async::{AsyncPgConnection, RunQueryDsl, SimpleAsyncConnection};
    use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig, RecyclingMethod};
    use diesel_async::pooled_connection::deadpool::{BuildError, Pool};
    use futures::Stream;
    use peak_alloc::PeakAlloc;
    use tokio::sync::Mutex;
    use zip::ZipArchive;

    use batch_processing::tokio::step::AsyncStepRunner;
    use batch_processing::tokio::step::complex_step::{AsyncComplexStepBuilder, ComplexStepBuilderTrait};
    use batch_processing::tokio::step::step_builder::AsyncStepBuilderTrait;

    #[global_allocator]
    static PEAK_ALLOC: PeakAlloc = PeakAlloc;
    macro_rules! resource_file {($fname:expr) => (
      concat!(env!("CARGO_MANIFEST_DIR"), "/tests/resources/", $fname) // assumes Linux ('/')!
    )}

    diesel::table! {
        car_prices (id) {
            id -> Serial,
            year -> Nullable<Integer>,
            make -> Nullable<VarChar>,
            model -> Nullable<VarChar>,
            trim -> Nullable<VarChar>,
            body -> Nullable<VarChar>,
            transmission -> Nullable<VarChar>,
            vin -> Nullable<VarChar>,
            state -> Nullable<VarChar>,
            condition -> Nullable<Integer>,
            odometer -> Nullable<Integer>,
            color -> Nullable<VarChar>,
            interior -> Nullable<VarChar>,
            seller -> Nullable<VarChar>,
            nmr -> Nullable<Integer>,
            sellingprice -> Nullable<Integer>,
            saledate -> Nullable<Date>,
        }
    }

    #[derive(Insertable, Queryable)]
    #[diesel(table_name = car_prices, primary_key(id))]
    pub struct CarPrice {
        pub id: Option<i32>,
        pub year: Option<i32>,
        pub make: Option<String>,
        pub model: Option<String>,
        pub trim: Option<String>,
        pub body: Option<String>,
        pub transmission: Option<String>,
        pub vin: Option<String>,
        pub state: Option<String>,
        pub condition: Option<i32>,
        pub odometer: Option<i32>,
        pub color: Option<String>,
        pub interior: Option<String>,
        pub seller: Option<String>,
        pub nmr: Option<i32>,
        pub sellingprice: Option<i32>,
        pub saledate: Option<NaiveDate>,
    }

    async fn get_pool() -> Result<Pool<AsyncPgConnection>, BuildError> {
        let pg_url = "postgres://postgres:postgres@localhost:5432/postgres";
        let mut config: ManagerConfig<AsyncPgConnection> = ManagerConfig::default();
        config.recycling_method = RecyclingMethod::Fast;
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(pg_url, config);
        let mut timeout = Timeouts::new();
        timeout.recycle = Some(std::time::Duration::from_secs(10));

        let pool = Pool::builder(config)
            .max_size(2)
            .build();

        if let Err(e) = pool {
            return Err(e);
        };

        let pool = pool.unwrap();
        Ok(pool)
    }

    fn unzip_csv_file() -> Option<String> {
        let output_path = "/tmp/car_prices.csv".to_string();
        let exist_file = Path::new("/tmp/car_prices.csv").exists();
        if exist_file {
            return Some(output_path);
        }

        let zip_path = resource_file!("car_prices.zip");
        let zip_file = File::open(zip_path).unwrap();

        let mut archiver = ZipArchive::new(zip_file).unwrap();

        for i in 0..archiver.len() {
            let mut file = archiver.by_index(i).unwrap();
            if file.name().ends_with(".csv") {
                let mut output_file = File::create(output_path.clone()).unwrap();
                io::copy(&mut file, &mut output_file).unwrap();
                return Some(output_path);
            }
        }

        return None;
    }

    #[tokio::test]
    #[ignore]
    async fn test_transfer_data_complex_step() {
        // enable_test_log();
        let pool = Arc::new(get_pool().await.expect("Error creating pool"));
        let create_table_car_prices = include_str!("../../../migrations/car_prices.sql");
        let mut conn = pool.get().await.expect("Error getting connection");
        conn.batch_execute(create_table_car_prices).await.expect("Error creating table");
        conn.batch_execute("TRUNCATE TABLE car_prices").await.expect("Error truncating table");
        let csv_file = unzip_csv_file();
        if let None = csv_file {
            panic!("CSV file not found");
        }
        let csv_path = csv_file.unwrap();
        let all_memory_usage = Arc::new(Mutex::new(Vec::new()));
        let final_memory_usage = Arc::clone(&all_memory_usage);
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
                            // let mut vec = vec_car_price.chunks(2000);
                            let mut conn = pool.get().await.expect("Error getting connection");
                            // while let Some(chunk) = vec.next() {
                                insert_into(car_prices::table)
                                    .values(vec_car_price)
                                    // .values(chunk)
                                    .execute(&mut conn)
                                    .await
                                    .expect("Error inserting data");
                            // }
                            let current_mem = PEAK_ALLOC.current_usage_as_mb() as i32;
                            all_memory_usage.lock().await.push(current_mem);
                        }
                    );
                }
            )
        ).chunk_size(2000);

        let step = step_builder.build();

        step.run().await;

        let all_memory_usage = final_memory_usage.lock().await;

        let max_memory_for_usage = 4;

        let max_memory = all_memory_usage.iter().max().unwrap();

        println!("Max memory: {}", max_memory);

        assert!(max_memory <= &max_memory_for_usage, "Memory usage is greater than expected")
    }
}