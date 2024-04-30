use bruss_config::CONFIGS;
use bruss_data::{BrussType, Path};
use crate::db::BrussData;
use super::db::{DBResponse, db_query_doc};
use mongodb::bson::doc;
use rocket_db_pools::Connection;


#[get("/path/<path>")]
pub async fn get_path<'a>(db: Connection<BrussData>, path: &'a str) -> DBResponse<Path> {
    Path::get_coll(&db.database(CONFIGS.db.get_db()))
        .find_one(doc!{"id": path}, None)
        .await?.into()
}

