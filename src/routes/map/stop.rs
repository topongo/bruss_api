use bruss_data::Stop;
use crate::db::BrussData;
use super::db::{DBResponse, DBQuery, db_query_get};
use mongodb::bson::Document;
use rocket_db_pools::Connection;


#[derive(FromForm)]
pub struct StopQuery<'r> {
    id: Option<u16>,
    #[field(name = "type")]
    ty: Option<&'r str>,
}

impl DBQuery for StopQuery<'_> {
    fn to_doc(&self) -> Document {
        let mut d = Document::new();
        if let Some(id) = self.id {
            d.insert("id", id as i32);
        }
        if let Some(ty) = self.ty {
            d.insert("ty", ty);
        }
        d
    }
}

#[get("/stops?<query..>")]
pub async fn get_stops(db: Connection<BrussData>, query: StopQuery<'_>) -> DBResponse<Vec<Stop>> {
    db_query_get(db, query).await
}

