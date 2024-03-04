use crate::configs::CONFIGS;
use crate::data::Area;
use crate::tt::ToBruss;
use rocket::{fairing::{self, AdHoc}, Build, Rocket};
use rocket_db_pools::Database;
use mongodb::{Client,bson::doc};
use std::error::Error;

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
            let areas_c = db.database(CONFIGS.db.get_db()).collection::<Area>("areas");

            if let Err(e) = areas_c.delete_many(doc! { }, None).await {
                return db_migrate_fail(rocket, &e, Some("cannot connect"))
            }
            let tt = CONFIGS.tt.client();
            match tt.get_areas().await {
                Ok(tt_areas) => {
                    let mut areas = Vec::new();
                    for a in tt_areas {
                        areas.push(a.to_bruss());
                    }
                    if let Err(e) = areas_c.insert_many(areas, None).await {
                        return db_migrate_fail(rocket, &e, None)
                    }
                }
                Err(e) => return db_migrate_fail(rocket, &e, Some("Error while retrieving areas from TT"))
            }
            match tt.get_routes().await {
                Ok(tt_routes) => {
                    let mut routes = Vec::new();
                    for r in tt_routes {
                        routes.push(r.to_bruss());
                    }
                }
                Err(e) => return db_migrate_fail(rocket, &e, Some("Error while retrieving routes from TT"))
            }
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

