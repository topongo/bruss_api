use bruss_data::Route;
use crate::db::BrussData;
use mongodb::bson::Document;
use rocket_db_pools::Connection;
use super::db::{db_query_get, DBQuery, DBResponse};

#[derive(FromForm)]
pub struct RouteQuery {
    id: Option<u16>,
    #[field(name = "type")]
    ty: Option<u16>,
    area: Option<u16>,
}

impl DBQuery for RouteQuery {
    fn to_doc(&self) -> Document {
        let mut d = Document::new();
        let RouteQuery { id, ty, area } = self;
        if let Some(id) = id { d.insert("id", *id as i32); }
        if let Some(ty) = ty { d.insert("type", *ty as i32); }
        if let Some(area) = area { d.insert("area", *area as i32); }
        d
    }
}

#[get("/routes?<query..>")]
pub async fn get_routes(db: Connection<BrussData>, query: RouteQuery) -> DBResponse<Vec<Route>> {
    db_query_get(db, query).await
}


