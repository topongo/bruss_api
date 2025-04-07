use bruss_data::Path;
use lazy_static::lazy_static;
use crate::db::BrussData;
use crate::response::ApiResponse;
use mongodb::bson::doc;
use rocket_db_pools::Connection;
use super::{pipeline::Pipeline, query::{DBInterface, UniformQueryable}};

#[get("/<paths>")]
pub async fn get(db: Connection<BrussData>, paths: &str) -> ApiResponse<Vec<Path>> {
    UniformQueryable::<Path>::query(&DBInterface(db), Pipeline::from(doc!{"id": {"$in": paths.split(",").collect::<Vec<&str>>()}})).await.into()
}

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get];
}

