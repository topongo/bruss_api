use crate::configs::CONFIGS;
use crate::data::{Area, FromTT, Route, Stop};
use tt::{TTArea, TTClient, TTRoute, TTStop, TTType, VecEndpoint};
use mongodb::Collection;
use rocket::{fairing::{self, AdHoc}, Build, Rocket};
use rocket_db_pools::Database;
use mongodb::{Client,bson::doc};
use serde::Serialize;
use std::error::Error;

#[derive(Database)]
#[database("bruss")]
pub struct BrussData(Client);
//
// async {
//     match areas_c.count_documents(doc!{},None).await {
//         Ok(n) => if n > 0 {
//             info!("retrieving areas data...");
//             match tt.get_areas().await {
//                 Ok(tt_areas) => {
//                     let mut areas = Vec::new();
//                     for a in tt_areas {
//                         areas.push(a.to_bruss());
//                     }
//                     if let Err(e) = areas_c.insert_many(areas, None).await {
//                         Some((Box::new(e) as Box<dyn Error>, None))
//                     } else {
//                         None
//                     }
//                 }
//                 Err(e) => Some((Box::new(e) as Box<dyn Error>, Some("Error while retrieving areas from TT")))
//             }
//         } else { None },
//         Err(e) => Some((Box::new(e) as Box<dyn Error>, Some("cannot count areas documents"))),
//     } 
// }

type FetchResult = Option<(Box<dyn Error>, Option<&'static str>)>;

async fn fetch_all_insert<I, O>(coll: Collection<O>) -> FetchResult 
    where O: FromTT<I> + Serialize, I: TTType, TTClient: VecEndpoint<I>
{
    match coll.count_documents(doc!{}, None).await {
        Ok(n) => if n > 0 {
            info!("retrieving {} data...", coll.name());
            // compiler needs all this junk...
            let res = <TTClient as VecEndpoint<I>>::request(&CONFIGS.tt.client()).await;
            match res {
                Ok(tt_data) => {
                    let mut data: Vec<O> = Vec::new();
                    for d in tt_data {
                        data.push(O::from_tt(d));
                    }
                    if let Err(e) = coll.insert_many(data, None).await {
                        Some((Box::new(e) as Box<dyn Error>, None))
                    } else {
                        None
                    }
                }
                Err(e) => Some((Box::new(e) as Box<dyn Error>, Some("cannot count documents")))
            } 
        } else { None }
        Err(e) => Some((Box::new(e) as Box<dyn Error>, Some("cannot count documents")))
    }
}

fn db_migrate_fail(rocket: Rocket<Build>, error: Box<dyn Error>, message: Option<&'static str>) -> fairing::Result {
    error!("cannot migrate database: {}", match message {
        Some(m) => format!("{}: {}", m, error),
        None => error.to_string()
    });
    Err(rocket)
}

async fn migrate(rocket: Rocket<Build>) -> fairing::Result {
    match BrussData::fetch(&rocket) {
        Some(db) => {
            info!("connecting to collections");
            let areas_c = db.database(CONFIGS.db.get_db()).collection::<Area>("areas");
            let routes_c = db.database(CONFIGS.db.get_db()).collection::<Route>("routes");
            let stops_c = db.database(CONFIGS.db.get_db()).collection::<Stop>("stops");

            match fetch_all_insert::<TTArea, Area>(areas_c).await {
                Some((e, msg)) => return db_migrate_fail(rocket, e, msg),
                None => {}
            }
            match fetch_all_insert::<TTRoute, Route>(routes_c).await {
                Some((e, msg)) => return db_migrate_fail(rocket, e, msg),
                None => {}
            }
            match fetch_all_insert::<TTStop, Stop>(stops_c).await {
                Some((e, msg)) => return db_migrate_fail(rocket, e, msg),
                None => {}
            }

            Ok(rocket)
        }
        None => panic!("misconfigured database, this behavior is wrong at compile time.") 
    }
}

pub fn db_init() -> AdHoc {
    AdHoc::on_ignite("Database connect", |rocket| async {
        rocket.attach(BrussData::init())
            .attach(AdHoc::try_on_ignite("Database migrate", migrate))
    })
}

