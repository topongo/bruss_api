use crate::configs::CONFIGS;
use crate::data::{Area, Route, Stop, ToBruss};
use crate::tt::{VecEndpoint, TTResult, TTClient};
use mongodb::Collection;
use rocket::{fairing::{self, AdHoc}, Build, Rocket};
use rocket_db_pools::Database;
use mongodb::{Client,bson::doc};
use serde::de::DeserializeOwned;
use tokio::join;
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

async fn fetch_all_insert<I, O>(coll: Collection<O>, coll_name: &'static str) -> Option<(Box<dyn Error>, Option<&'static str>)> 
    where O: Serialize, I: ToBruss<Output = O> + DeserializeOwned, TTClient: VecEndpoint<I>
{
    match coll.count_documents(doc!{}, None).await {
        Ok(n) => if n > 0 {
            info!("retrieving {} data...", coll_name);
            // compiler needs all this junk...
            let res = <TTClient as VecEndpoint<I>>::request(&CONFIGS.tt.client()).await;
            match res {
                Ok(tt_data) => {
                    let mut data: Vec<O> = Vec::new();
                    for d in tt_data {
                        data.push(d.to_bruss());
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
             
            // info!("deleting old data from collections");
            // if let Err(e) = areas_c.delete_many(doc! { }, None).await {
            //     return db_migrate_fail(rocket, &e, Some("cannot clean areas collection"))
            // }
            // if let Err(e) = routes_c.delete_many(doc! { }, None).await {
            //     return db_migrate_fail(rocket, &e, Some("cannot clean routes collection"))
            // }
            info!("creating tt client");
            let tt = CONFIGS.tt.client();
            
            type InitRes = Option<(Box<dyn Error>, Option<&'static str>)>;
            let (areas_r, routes_r, stops_r): (InitRes, InitRes, InitRes)  = join!(
                async {
                    match routes_c.count_documents(doc!{},None).await {
                        Ok(n) => if n > 0 {
                            info!("retrieving routes data...");
                            match tt.get_routes().await {
                                Ok(tt_routes) => {
                                    let mut routes = Vec::new();
                                    for a in tt_routes {
                                        routes.push(a.to_bruss());
                                    }
                                    if let Err(e) = routes_c.insert_many(routes, None).await {
                                        Some((Box::new(e) as Box<dyn Error>, None))
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => Some((Box::new(e) as Box<dyn Error>, Some("Error while retrieving routes from TT")))
                            }
                        } else { None },
                        Err(e) => Some((Box::new(e) as Box<dyn Error>, Some("cannot count routes documents"))),
                    } 
                },
                async {
                    match stops_c.count_documents(doc!{},None).await {
                        Ok(n) => if n > 0 {
                            info!("retrieving stops data...");
                            match tt.get_stops().await {
                                Ok(tt_stops) => {
                                    let mut stops = Vec::new();
                                    for a in tt_stops {
                                        stops.push(a.to_bruss());
                                    }
                                    if let Err(e) = stops_c.insert_many(stops, None).await {
                                        Some((Box::new(e) as Box<dyn Error>, None))
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => Some((Box::new(e) as Box<dyn Error>, Some("Error while retrieving stops from TT")))
                            }
                        } else { None },
                        Err(e) => Some((Box::new(e) as Box<dyn Error>, Some("cannot count stops documents"))),
                    } 
                }
            );

            if let Some((e, msg)) = areas_r {
                db_migrate_fail(rocket, e, msg)
            } else if let Some((e, msg)) = routes_r {
                db_migrate_fail(rocket, e, msg)
            } else if let Some((e, msg)) = stops_r {
                db_migrate_fail(rocket, e, msg)
            } else {
                Ok(rocket)
            }
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

