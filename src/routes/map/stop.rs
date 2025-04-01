use bruss_config::CONFIGS;
use bruss_data::{Route, Stop, Trip};
use lazy_static::lazy_static;
use tt::AreaType;
use crate::db::BrussData;
use super::{gen_area_getters, params::{Id, ParamQuery}, pipeline::Pipeline, query::{DBInterface, DBQuery, Queriable}, trip::TripQuery, FromStringFormField};
use mongodb::bson::{doc, Document};
use rocket_db_pools::Connection;
use crate::response::ApiResponse;
use rocket::{request::FromParam,form::Strict};


#[derive(FromForm)]
pub struct StopQuery {
    // id: Strict<Option<u16>>,
    #[field(name = "type")]
    ty: Strict<Option<FromStringFormField<AreaType>>>,
}

impl DBQuery for StopQuery {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        if let Some(ty) = self.ty.into_inner() { d.insert::<_, &'static str>("type", ty.into_inner().into()); }
        d
    }
}

gen_area_getters!(Stop, StopQuery, u16);


#[get("/<area_type>/<id>/trips?<limit>&<skip>&<query..>")]
async fn get_trips(
    db: Connection<BrussData>,
    area_type: Result<Id<FromStringFormField<AreaType>>, <Id<FromStringFormField<AreaType>> as FromParam<'_>>::Error>,
    id: Result<Id<u16>, <Id<u16> as FromParam<'_>>::Error>, 
    query: rocket::form::Result<'_, Strict<TripQuery>>,
    limit: Option<u32>,
    skip: Option<u32>,
) -> ApiResponse<Vec<Trip>> {
    let id = id?.value();
    // query?.into_inner().to_doc_stop(id as u16, area_type?.value().inner);
    // d.insert("type", );
    // let query = query?.into_inner().into_doc_stop(id as u16, area_type?.value().inner);
    let query = doc! {
        "type": area_type?.value().inner.to_string(),
        "times": { "$elemMatch": { "stop": id } },
    };
    let pipeline = Pipeline::new(query)
        .pre_sort(doc!{"$project": {
                "id": 1,
                "delay": 1,
                "direction": 1,
                "next_stop": 1,
                "last_stop": 1,
                "bus_id": 1,
                "route": 1,
                "headsign": 1,
                "path": 1,
                "times": 1,
                "type": 1,
                "sequence": 1,
                "arrival_time": format!("$times.{}.arrival", id), 
            }}
        )
        .limit(limit)
        .skip(skip)
        .sort(doc!{"arrival_time": 1});
    Queriable::<QueryResult<Trip>>::query(&DBInterface(db), pipeline).await.into()
}

#[get("/<area_type>/<id>/routes?<limit>&<skip>")]
async fn get_routes(
    db: Connection<BrussData>,
    area_type: Result<Id<FromStringFormField<AreaType>>, <Id<FromStringFormField<AreaType>> as FromParam<'_>>::Error>,
    id: Result<Id<u16>, <Id<u16> as FromParam<'_>>::Error>,
    limit: Option<u32>,
    skip: Option<u32>,
) -> ApiResponse<Vec<Route>> {
    let id = id?.value();
    let ty: &str = area_type?.value().into_inner().into();
    
    let route_ids = db
        .database(CONFIGS.db.get_db())
        .collection::<i32>("trips")
        .distinct("route", doc!{ "type": ty, "$or": [ { format!("times.{}", id): { "$exists": true } }, { format!("times.{}", id): { "$exists": true } } ] }, None)
        .await?;
        
    Queriable::<QueryResult<Route>>::query(&DBInterface(db), Pipeline::new(doc!{"id": {"$in": route_ids}}).limit(limit).skip(skip)).await.into()
}

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get, get_opts, get_trips, get_routes];
}

