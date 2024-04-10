use bruss_data::Segment;
use rocket::serde::json::Json;
use crate::db::BrussData;
use mongodb::bson::{Document,doc};
use rocket_db_pools::Connection;
use super::db::{db_query_json, DBResponse, DBQuery};
use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
#[derive(FromForm)]
pub struct SegmentQuery {
    stops: Option<Vec<(u16, u16)>>,
    #[field(name = "type")]
    ty: Option<u16>
}

impl DBQuery for SegmentQuery {
    fn to_doc(&self) -> Document {
        let mut d = Document::new();
        let Self { ty, stops } = self;

        match stops {
            Some(stops) => if stops.len() > 0 {
                d.insert("$or", stops.iter()
                    .map(|(s1, s2)| doc!{"from": *s1 as i32, "to": *s2 as i32})
                    .collect::<Vec<Document>>()
                );
            }
            None => {},
        }
        // info!("id field: {:?}", segs);
        // if let Some(id) = id { d.insert("id", id.iter().map(|v| *v as i32).collect::<Vec<i32>>() ); }
        if let Some(ty) = ty { d.insert("type", *ty as i32); }
        info!("document: {d:?}");
        d
    }
}

#[post("/segments", format = "json", data = "<query>")]
pub async fn get_segments(db: Connection<BrussData>, query: Json<SegmentQuery>) -> DBResponse<Vec<Segment>> {
    db_query_json(db, query).await
}

