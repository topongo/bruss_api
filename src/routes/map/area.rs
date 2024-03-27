use bruss_data::Area;
use crate::db::BrussData;
use super::getter::{GetResponse, GetterQuery, get};
use mongodb::bson::Document;
use rocket_db_pools::Connection;


#[derive(FromForm)]
pub struct AreaQuery<'r> {
    id: Option<u16>,
    #[field(name = "type")]
    ty: Option<&'r str>,
}

impl GetterQuery for AreaQuery<'_> {
    fn to_doc(self) -> Document {
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

#[get("/areas?<query..>")]
pub async fn get_areas(db: Connection<BrussData>, query: AreaQuery<'_>) -> GetResponse<Vec<Area>> {
    get(db, query).await
}

