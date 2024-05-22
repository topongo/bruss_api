use bruss_data::{Stop, Trip};
use lazy_static::lazy_static;
use crate::db::BrussData;
use super::{gen_area_getters, query::{DBInterface, DBQuery, Queriable}, params::{Id, ParamQuery}, AreaTypeWrapper, trip::TripQuery};
use mongodb::bson::Document;
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

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get, get_opts, get_trips];
}

