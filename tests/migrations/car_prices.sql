create table if not exists car_prices(
                           id serial primary key,
                           year int,
                           make text,
                           model text,
                           trim text,
                           body text,
                           transmission text,
                           vin text,
                           state text,
                           condition int,
                           odometer int,
                           color text,
                           interior text,
                           seller text,
                           mmr int,
                           sellingprice int,
                           saledate date
);