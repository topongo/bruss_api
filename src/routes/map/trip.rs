use bruss_data::Trip;
use rocket::form::{self, Errors, FromForm, FromFormField};
use tt::AreaType;
use crate::db::BrussData;
use mongodb::bson::{doc, Document};
use rocket_db_pools::Connection;
use super::{db::{db_query_get, DBQuery, DBResponse}, AreaTypeWrapper};

#[derive(FromForm)]
pub struct TripStopQuery {
    id: Option<String>,
    #[field(name = "type")]
    ty: AreaTypeWrapper,
    stop: u16,
}

#[derive(FromForm)]
pub struct TripRouteQuery {
    id: Option<String>,
    #[field(name = "type")]
    ty: AreaTypeWrapper,
    route: u16,
}

impl DBQuery for TripStopQuery {
    fn to_doc(&self) -> Document {
        let TripStopQuery { id, ty, stop } = self;
        let mut d = Document::new();
        d.insert(format!("times.{}", stop), doc!{"$exists": true});
        d.insert::<&'static str, &'static str>("type", ty.inner.into());
        if let Some(id) = id { d.insert("id", id.clone()); }
        d
    }
}

impl DBQuery for TripRouteQuery {
    fn to_doc(&self) -> Document {
        let TripRouteQuery { id, ty, route } = self;
        let mut d = Document::new();
        d.insert("route", *route as i32);
        d.insert::<&'static str, &'static str>("type", ty.inner.into());
        if let Some(id) = id { d.insert("id", id.clone()); }
        d 
    }
}

#[get("/trips_stop?<query..>")]
pub async fn get_trips_stop(db: Connection<BrussData>, query: TripStopQuery) -> DBResponse<Vec<Trip>> {
    db_query_get(db, query).await
}

#[get("/trips_route?<query..>")]
pub async fn get_trips_route(db: Connection<BrussData>, query: TripRouteQuery) -> DBResponse<Vec<Trip>> {
    db_query_get(db, query).await
}


