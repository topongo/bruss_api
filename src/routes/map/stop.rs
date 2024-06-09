use bruss_config::CONFIGS;
use bruss_data::{BrussType, Route, Stop, Trip};
use lazy_static::lazy_static;
use crate::db::BrussData;
use super::{gen_area_getters, query::{DBInterface, DBQuery, Queriable}, params::{Id, ParamQuery}, AreaTypeWrapper, trip::TripQuery};
use mongodb::bson::{doc, Document};
use rocket_db_pools::Connection;
use crate::response::ApiResponse;
use rocket::{request::FromParam,form::Strict};


#[derive(FromForm)]
pub struct StopQuery {
    // id: Strict<Option<u16>>,
    #[field(name = "type")]
    ty: Strict<Option<AreaTypeWrapper>>,
}

impl DBQuery for StopQuery {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        if let Some(ty) = self.ty.into_inner() { d.insert::<_, &'static str>("type", ty.into()); }
        d
    }
}

gen_area_getters!(Stop, StopQuery, u16);


#[get("/<area_type>/<id>/trips?<query..>")]
async fn get_trips(
    db: Connection<BrussData>,
    area_type: Result<Id<AreaTypeWrapper>, <Id<AreaTypeWrapper> as FromParam<'_>>::Error>, 
    id: Result<Id<u16>, <Id<u16> as FromParam<'_>>::Error>, 
    query: rocket::form::Result<'_, Strict<TripQuery>>
) -> ApiResponse<Vec<Trip>> {
    let id = id?.value();
    // query?.into_inner().to_doc_stop(id as u16, area_type?.value().inner);
    // d.insert("type", );
    Queriable::<Vec<Trip>>::query(&DBInterface(db), query?.into_inner().to_doc_stop(id as u16, area_type?.value().inner)).await.into()
}

#[get("/<area_type>/<id>/routes")]
async fn get_routes(
    db: Connection<BrussData>,
    area_type: Result<Id<AreaTypeWrapper>, <Id<AreaTypeWrapper> as FromParam<'_>>::Error>, 
    id: Result<Id<u16>, <Id<u16> as FromParam<'_>>::Error>
) -> ApiResponse<Vec<Route>> {
    let id = id?.value();
    let ty: &str = area_type?.value().into();
    
    let route_ids = db
        .database(CONFIGS.db.get_db())
        .collection::<i32>("trips")
        .distinct("route", doc!{ "type": ty, "$or": [ { format!("times.{}", id): { "$exists": true } }, { format!("times.{}", id): { "$exists": true } } ] }, None)
        .await?;
        
    Queriable::<Vec<Route>>::query(&DBInterface(db), doc!{"id": {"$in": route_ids}}).await.into()
}

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get, get_opts, get_trips, get_routes];
}

