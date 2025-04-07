use bruss_data::{Route, Schedule};
use lazy_static::lazy_static;
use tt::AreaType;
use crate::{db::BrussData, routes::map::{query::Queryable, trip::TripCross}};
use mongodb::bson::{doc, Document};
use rocket_db_pools::Connection;
use super::{gen_generic_getters, params::{Id,ParamQuery}, query::{DBInterface, DBQuery}, trip::MultiTripQuery, FromStringFormField};
use crate::response::ApiResponse;
use rocket::form::Strict;
use rocket::request::FromParam;
use super::pipeline::Pipeline;

#[derive(FromForm,Debug)]
pub struct RouteQuery {
    #[field(name = "type")]
    ty: Strict<Option<FromStringFormField<AreaType>>>,
    id: Strict<Option<Vec<u16>>>,
    area: Strict<Option<u16>>,
}

impl DBQuery for RouteQuery {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        error!("{self:?}");
        let RouteQuery { ty, area, id } = self;
        // if let Some(id) = id { d.insert("id", id as i32); }
        if let Some(ty) = ty.into_inner() { d.insert("area_ty", ty.into_bson()); }
        if let Some(area) = area.into_inner() { d.insert("area", area as i32); }
        if let Some(id) = id.into_inner() { d.insert("id", doc!{"$in": id.iter().map(|v| *v as i32).collect::<Vec<i32>>()}); }
        d
    }
}

gen_generic_getters!(Route, RouteQuery, u16);

#[get("/<id>/trips?<limit>&<skip>&<query..>")]
async fn get_trips(
    db: Connection<BrussData>,
    id: Result<Id<u16>, <Id<u16> as FromParam<'_>>::Error>, 
    query: rocket::form::Result<'_, Strict<MultiTripQuery>>,
    limit: Option<u32>,
    skip: Option<u32>,
) -> ApiResponse<Vec<TripCross>> {
    let id = id?.value();

    let pipeline = query?
        .into_inner()
        .into_pipeline_route(id as u16, skip, limit);
    println!("Pipeline: {pipeline}");

    Queryable::<TripCross, Schedule>::query(&DBInterface(db), pipeline).await.into()
}

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get, get_opts, get_trips];
}
