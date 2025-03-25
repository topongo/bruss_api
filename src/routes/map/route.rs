use bruss_data::{Route, Trip};
use lazy_static::lazy_static;
use crate::db::BrussData;
use mongodb::bson::{doc, Document};
use rocket_db_pools::Connection;
use super::{gen_generic_getters,trip::TripQuery,AreaTypeWrapper,query::{Queriable,DBInterface,DBQuery},params::{Id,ParamQuery}};
use crate::response::ApiResponse;
use rocket::form::Strict;
use rocket::request::FromParam;
use super::pipeline::Pipeline;

#[derive(FromForm,Debug)]
pub struct RouteQuery {
    #[field(name = "type")]
    ty: Strict<Option<AreaTypeWrapper>>,
    id: Strict<Option<Vec<u16>>>,
    area: Strict<Option<u16>>,
}

impl DBQuery for RouteQuery {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        error!("{self:?}");
        let RouteQuery { ty, area, id } = self;
        // if let Some(id) = id { d.insert("id", id as i32); }
        if let Some(ty) = ty.into_inner() { d.insert::<_, &'static str>("area_ty", ty.into()); }
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
    query: rocket::form::Result<'_, Strict<TripQuery>>,
    limit: Option<u32>,
    skip: Option<u32>,
) -> ApiResponse<Vec<Trip>> {
    let id = id?.value();
    let pipeline = Pipeline::from(query?.into_inner().into_doc_route(id as u16))
        .limit(limit)
        .skip(skip);
    Queriable::<QueryResult<Trip>>::query(&DBInterface(db), pipeline).await.into()
}

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get, get_opts, get_trips];
}
