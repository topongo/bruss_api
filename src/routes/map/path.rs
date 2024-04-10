use bruss_data::Path;
use crate::db::BrussData;
use super::db::{DBResponse, db_query_doc};
use mongodb::bson::doc;
use rocket_db_pools::Connection;


#[get("/path/<path>")]
pub async fn get_path<'a>(db: Connection<BrussData>, path: &'a str) -> DBResponse<Vec<Path>> {
    db_query_doc(db, doc!{"id": path}).await
}

