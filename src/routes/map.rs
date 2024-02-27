use rocket::serde::json::Json;
use crate::data::{Area, AreaType};
use tokio_postgres::NoTls;
use tokio_postgres::row::Row;
use crate::configs::BrussConfig;
use rocket::catcher::Result;

fn db_areas() -> Vec<Area> {
   vec![] 
}

#[get("/areas")]
pub async fn get_areas() -> Json<Vec<Area>> {
    let config = BrussConfig::from_file("/home/topongo/fast/documents/uni/internship/bruss/app/api/config.toml").get_db_configs();
    let (client, connection) = config.connect(NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows: Vec<tokio_postgres::row::Row> = client.query("SELECT id, ty, label FROM areas", &[]).await.unwrap();

    let areas: Vec<Area> = serde_postgres::from_rows(&rows).unwrap();
    Json(areas)
}

