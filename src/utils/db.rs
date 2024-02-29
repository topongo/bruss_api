use crate::data::Area;
use crate::configs::CONFIGS;
use reqwest::get;
use rocket::Config;

/// Assume database is empty, initialize new database
pub fn build_db() {
    let figment = Config::figment();
    let conf = Config::from(figment);
    println!("Config: {conf:?}");
}

/// Clean existing database and initialize it.
pub fn clean_and_build_db() {
    todo!()
}

