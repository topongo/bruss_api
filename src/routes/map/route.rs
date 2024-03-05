use crate::data::Route;
use crate::db::BrussData;
use mongodb::bson::Document;
use rocket_db_pools::Connection;
use super::getter::{get, GetResponse, GetterQuery};

#[derive(FromForm)]
pub struct RouteQuery {
    id: Option<u16>,
    #[field(name = "type")]
    ty: Option<u16>
}

impl GetterQuery for RouteQuery {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        let RouteQuery { id, ty } = self;
        if let Some(id) = id { d.insert("id", id as i32); }
        if let Some(ty) = ty { d.insert("type", ty as i32); }
        d
    }
}

#[get("/routes?<query..>")]
pub async fn get_routes(db: Connection<BrussData>, query: RouteQuery) -> GetResponse<Vec<Route>> {
    get(db, query).await
}
