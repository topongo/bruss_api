use rocket::futures::StreamExt;
use rocket::serde::json::Json;
use crate::data::{Area, AreaType};
use crate::configs::CONFIGS;
use rocket::catcher::Result;
use mongodb::{bson::doc, Client};
use futures::stream::TryStreamExt;

fn db_areas() -> Vec<Area> {
   vec![] 
}

#[get("/areas")]
pub async fn get_areas() -> Json<Vec<Area>> {
    let db_config = CONFIGS.db.gen_mongodb_options();
    let client = Client::with_options(db_config).unwrap();

    let db = client.database(CONFIGS.db.get_db());

    let areas_c = db.collection::<Area>("areas");
    let cursor = areas_c.find(doc! {}, None).await.unwrap();

    Json(cursor.try_collect::<Vec<Area>>().await.unwrap())
}

