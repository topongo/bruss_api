#![cfg(test)]

use bruss_data::BrussType;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use mongodb::bson::doc;
use tt::{AreaType, TTTrip, TTType};

#[tokio::test]
async fn test_trips() {
    use tt::{TTClient,RequestOptions,TripQuery};
    use bruss_config::CONFIGS;
    use mongodb::Client;
    use bruss_data::Trip;

    let client: TTClient = CONFIGS.tt.client();
    let db = Client::with_options(CONFIGS.db.gen_mongodb_options()).unwrap().database(CONFIGS.db.get_db());

    let t = Trip::get_coll(&db)
        .find_one(doc! { }, None)
        .await
        .unwrap()
        .unwrap();
    println!("{}", t.id);
    
    let time = t.times.iter().enumerate().find(|(n, _)| *n == 0).unwrap().1.1;
    println!("{:?}", time);

    let time = Local::now().date_naive().and_time(time.arrival);

    let trips: Vec<TTTrip> = client.request_opt(Some(RequestOptions::new().query(TripQuery { route_id: t.route, ty: t.ty, limit: 1, time }))).await.unwrap();
    let ids = trips.into_iter().map(|t| t.id).collect::<Vec<_>>();
    println!("{ids:?}");
    assert!(ids.contains(&t.id))
}
