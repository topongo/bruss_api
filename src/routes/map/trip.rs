use rocket::form::FromForm;
use tt::AreaType;
use mongodb::bson::{doc, Document};

#[derive(FromForm)]
pub struct TripQuery {
    id: Option<String>,
}

impl TripQuery {
    pub fn to_doc_stop(self, stop: u16, ty: AreaType) -> Document {
        let Self { id } = self;
        let mut d = Document::new();
        d.insert(format!("times.{}", stop), doc!{"$exists": true});
        d.insert::<&'static str, &'static str>("type", ty.into());
        if let Some(id) = id { d.insert("id", id.clone()); }
        d
    }

    pub fn to_doc_route(self, route: u16) -> Document {
        let Self { id } = self;
        let mut d = Document::new();
        d.insert("route", route as i32);
        if let Some(id) = id { d.insert("id", id.clone()); }
        d 
    }
}

