use crate::configs::CONFIGS;
use crate::data::{Area, Route, Stop, ToBruss};
use rocket::{fairing::{self, AdHoc}, Build, Rocket};
use rocket_db_pools::Database;
use mongodb::{Client,bson::doc};
use tokio::join;
use tokio::time::sleep;
use std::error::Error;
use std::time::Duration;

#[derive(Database)]
#[database("bruss")]
pub struct BrussData(Client);


fn db_migrate_fail(rocket: Rocket<Build>, error: &dyn Error, message: Option<&'static str>) -> fairing::Result {
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
            
            let (areas_r, routes_r, stops_r): (Option<fairing::Result>, Option<fairing::Result>, Option::<fairing::Result>)  = join!(
                async {
                    match areas_c.count_documents(doc!{},None).await {
                        Ok(n) => if n > 0 {
                            info!("retrieving areas data...");
                            match tt.get_areas().await {
                                Ok(tt_areas) => {
                                    let mut areas = Vec::new();
                                    for a in tt_areas {
                                        areas.push(a.to_bruss());
                                    }
                                    if let Err(e) = areas_c.insert_many(areas, None).await {
                                        Some(db_migrate_fail(rocket, &e, None))
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => Some(db_migrate_fail(rocket, &e, Some("Error while retrieving areas from TT")))
                            }
                        } else { None },
                        Err(e) => Some(db_migrate_fail(rocket, &e, Some("cannot count areas documents"))),
                    } 
                },
                async {
                    match areas_c.count_documents(doc!{},None).await {
                        Ok(n) => if n > 0 {
                            info!("retrieving areas data...");
                            match tt.get_areas().await {
                                Ok(tt_areas) => {
                                    let mut areas = Vec::new();
                                    for a in tt_areas {
                                        areas.push(a.to_bruss());
                                    }
                                    if let Err(e) = areas_c.insert_many(areas, None).await {
                                        Some(db_migrate_fail(rocket, &e, None))
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => Some(db_migrate_fail(rocket, &e, Some("Error while retrieving areas from TT")))
                            }
                        } else { None },
                        Err(e) => Some(db_migrate_fail(rocket, &e, Some("cannot count areas documents"))),
                    }
                },
                async {
                    match areas_c.count_documents(doc!{},None).await {
                        Ok(n) => if n > 0 {
                            info!("retrieving areas data...");
                            match tt.get_areas().await {
                                Ok(tt_areas) => {
                                    let mut areas = Vec::new();
                                    for a in tt_areas {
                                        areas.push(a.to_bruss());
                                    }
                                    if let Err(e) = areas_c.insert_many(areas, None).await {
                                        Some(db_migrate_fail(rocket, &e, None))
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => Some(db_migrate_fail(rocket, &e, Some("Error while retrieving areas from TT")))
                            }
                        } else { None },
                        Err(e) => Some(db_migrate_fail(rocket, &e, Some("cannot count areas documents"))),
                    }
                });
            // if areas_c.count_documents(doc! {}, None).await
            
            info!("retrieving routes data");
            match tt.get_routes().await {
                Ok(tt_routes) => {
                    info!("loading routes into db...");
                    let mut routes = Vec::new();
                    for r in tt_routes {
                        routes.push(r.to_bruss());
                    }
                    if let Err(e) = routes_c.insert_many(routes, None).await {
                        return db_migrate_fail(rocket, &e, None)
                    }
                }
                Err(e) => return db_migrate_fail(rocket, &e, Some("Error while retrieving routes from TT"))
            }
            info!("retrieving routes data");
            match tt.get_stops().await {
                Ok(tt_stops) => {
                    info!("loading stops into db...");
                    let mut stops = Vec::new();
                    for r in tt_stops {
                        stops.push(r.to_bruss());
                    }
                    if let Err(e) = stops_c.insert_many(stops, None).await {
                        return db_migrate_fail(rocket, &e, None)
                    }
                }
                Err(e) => return db_migrate_fail(rocket, &e, Some("Error while retrieving stops from TT"))
            }
            info!("database setup done!")
        }
        None => panic!("misconfigured database, this behavior is wrong at compile time.") 
    }
       
    Ok(rocket)
}

pub fn db_init() -> AdHoc {
    AdHoc::on_ignite("Database connect", |rocket| async {
        rocket.attach(BrussData::init())
            .attach(AdHoc::try_on_ignite("Database migrate", migrate))
    })
}

