use rocket::serde::json::Json;
use crate::data::{Area, AreaType};
use tokio_postgres::{connect,NoTls};
use tokio;

fn db_areas() -> Vec<Area> {
   vec![] 
}

#[get("/areas")]
pub async fn get_areas() -> Json<Vec<Area>> {
    let (client, connection) = connect("host=localhost user=postgres password=culone database=bruss", NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("{:?}", client.query("\\connect bruss; SELECT * FROM areas", &[]).await.unwrap());
    Json(db_areas())
}

