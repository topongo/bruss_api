use bruss_config::CONFIGS;
use bruss_data::{BrussType, FromTT, Route, Trip};
use lazy_static::lazy_static;
use rocket::request::FromParam;
use rocket_db_pools::Connection;
use serde::{Serialize,Deserialize};
use mongodb::bson::{doc, Document};
use tt::{AreaType, TTTrip};

use crate::{db::BrussData, response::{ApiError, ApiResponse}, routes::map::query::{DBInterface, DBQuery, Queriable}};

#[derive(Debug, Serialize)]
pub struct TripTracking {
    delay: i32,
    last_stop: u16,
    next_stop: u16,
    area: AreaType,
    bus_id: Option<u16>,
}

impl From<(AreaType, Trip)> for TripTracking {
    fn from(value: (AreaType, Trip)) -> Self {
        let (area, value) = value;
        Self {
            delay: value.delay,
            last_stop: value.last_stop,
            next_stop: value.next_stop,
            area,
            bus_id: value.bus_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TripId(String);

impl FromParam<'_> for TripId {
    type Error = ();

    fn from_param(param: &'_ str) -> Result<Self, Self::Error> {
        Ok(TripId(param.to_string()))
    }

}

impl DBQuery for TripId {
    fn to_doc(self) -> Document {
        doc!{"id": self.0}
    }
}

#[get("/trip/<trip_id>")]
pub async fn get_trip(db: Connection<BrussData>, trip_id: TripId) -> ApiResponse<TripTracking> {
    let db = DBInterface(db);
    match Queriable::<Option<Trip>>::query(&db, trip_id.into()).await? {
        Some(t) => {
            let cli = CONFIGS.tt.client();
            let trip_new: TTTrip = cli.request_one(t.id.clone())
                .await?;
            let trip_new = Trip::from_tt(trip_new);
            let t = t.merge(trip_new);
            let ty = match Queriable::<Option<Route>>::query(&db, doc!{"id": t.route as i32}.into()).await? {
                Some(r) => r.area_ty,
                None => return ApiResponse::Error(ApiError::NotFound),
            };
            ApiResponse::Ok((ty, t).into(), None)
        },
        None => ApiResponse::Error(ApiError::NotFound),
    }
}


lazy_static! {
    pub static ref ROUTES: Vec<rocket::Route> = routes![get_trip];
}
