#![feature(try_trait_v2)]
// #![feature(inherent_associated_types)]
#[macro_use] 
extern crate rocket;

use rocket::fairing::AdHoc;
use crate::db::BrussData;
use rocket_db_pools::Database;

mod routes;
mod db;
mod cors;
#[cfg(test)]
mod tests;
mod response;


#[get("/")]
fn welcome_app() -> &'static str {
    "Welcome to the Bruss App!"
}

#[get("/api/v1")]
fn welcome_api() -> &'static str {
    "Welcome to the Bruss API!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![welcome_api, welcome_app])
        .mount("/api/v1/map/area", routes::map::area::ROUTES.clone())
        .mount("/api/v1/map/route", routes::map::route::ROUTES.clone())
        .mount("/api/v1/map/stop", routes::map::stop::ROUTES.clone())
        .mount("/api/v1/map/path", routes::map::path::ROUTES.clone())
        .mount("/api/v1/map/segment", routes::map::segment::ROUTES.clone())
        .mount("/api/v1/map/trip", routes::map::trip::ROUTES.clone())
            // routes::map::,
            // routes::map::get_route_opt,
            // routes::map::get_segments,
            // // routes::map::get_segments_poly,
            // routes::map::get_stops,
            // routes::map::get_trips_route,
            // routes::map::get_trips_stop,
            // routes::map::get_path,
        .mount("/api/v1/map", routes![routes::options])
        .mount("/api/v1/tracking/", routes::tracking::ROUTES.clone())
        .register("/api/v1/", catchers![
            response::api_catch_default,
            response::api_catch_404,
        ])
        .attach(AdHoc::on_ignite("Database connect", |rocket| async {
            rocket.attach(BrussData::init())
            // .attach(AdHoc::try_on_ignite("Database migrate", migrate))
        }))
        .attach(cors::CORS)
}

