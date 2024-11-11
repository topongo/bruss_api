use bruss_data::Path;
use lazy_static::lazy_static;
use crate::db::BrussData;
use crate::response::ApiResponse;
use mongodb::bson::doc;
use rocket_db_pools::Connection;

use super::{pipeline::Pipeline, query::{DBInterface, Queriable}};

#[get("/<paths>")]
pub async fn get<'a>(db: Connection<BrussData>, paths: &'a str) -> ApiResponse<Vec<Path>> {
    Queriable::<Vec<Path>>::query(&DBInterface(db), Pipeline::from(doc!{"id": {"$in": paths.split(",").collect::<Vec<&'a str>>()}})).await.into()
}

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get];
}

