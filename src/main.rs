#[macro_use] 
extern crate rocket;

mod routes;
mod data;
mod configs;
mod utils;

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
        .mount("/api/v1/map/", routes![
            routes::map::get_areas
        ])
        .mount("/api/v1/tracking/", routes![
        ])
}
