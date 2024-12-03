use std::collections::HashMap;

use bruss_config::CONFIGS;
use bruss_data::{FromTT, Route, Trip};
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use rocket::request::FromParam;
use rocket_db_pools::Connection;
use serde::{Serialize,Deserialize};
use mongodb::bson::{doc, Document};
use tt::{AreaType, TTTrip};
use crate::{db::BrussData, response::{ApiError, ApiResponse}, routes::map::query::{DBInterface, DBQuery, Queriable, QueryResult}};

#[derive(Debug, Serialize, Deserialize)]
pub struct TripTracking {
    id: String,
    delay: i32,
    last_stop: u16,
    next_stop: u16,
    area: Option<AreaType>,
    bus_id: Option<u16>,
}

impl TripTracking {
    fn error(id: String) -> Self {
        Self {
            id,
            delay: 0,
            last_stop: 0,
            next_stop: 0,
            area: None,
            bus_id: None,
        }
    }
}

impl From<(AreaType, Trip)> for TripTracking {
    fn from(value: (AreaType, Trip)) -> Self {
        let (area, value) = value;
        Self {
            id: value.id,
            delay: value.delay,
            last_stop: value.last_stop,
            next_stop: value.next_stop,
            area: Some(area),
            bus_id: value.bus_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TripId(Vec<String>);

impl FromParam<'_> for TripId {
    type Error = ();

    fn from_param(param: &'_ str) -> Result<Self, Self::Error> {
        Ok(TripId(param.split(',').map(str::to_string).collect::<Vec<String>>()))
    }

}

impl DBQuery for TripId {
    fn to_doc(self) -> Document {
        doc!{"id": doc!{"$in": self.0}}
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TripUpdate {
    #[serde(flatten)]
    tracking: TripTracking,
    #[serde(with = "chrono::serde::ts_seconds")]
    updated: DateTime<Utc>,
}

impl TripUpdate {
    async fn get_by_ids(db: &DBInterface, id: Vec<String>) -> Result<Vec<Self>, mongodb::error::Error> {
        let now = Utc::now();
        let coll = db.0.database(CONFIGS.db.get_db())
            .collection::<TripUpdate>("trip_updates");
        let updates: Vec<TripUpdate> = coll
            .find(doc!{"id": doc!{"$in": id}}, None)
            .await?
            .try_collect()
            .await?;

        let cli = CONFIGS.tt.client();
        let mut out = vec![];
        let mut updated = vec![];
        for u in updates.into_iter() {
            if now - u.updated > chrono::Duration::minutes(5) {
                let TripUpdate { tracking: t, updated: upd } = u;
                let trip_new: TTTrip = match cli.request_one(t.id.clone()).await {
                    Err(_) => {
                        // keep updated to old date, so it will be updated next time
                        out.push(TripUpdate {
                            tracking: TripTracking::error(t.id),
                            updated: upd,
                        });
                        continue;
                    }
                    Ok(tt) => tt,
                };
                let trip_new = Trip::from_tt(trip_new);
                updated.push(TripUpdate {
                    tracking: TripTracking::from((t.area.unwrap(), trip_new)),
                    updated: now,
                })
            } else {
                out.push(u);
            }
        }
        
        for u in updated.into_iter() {
            coll
                .replace_one(doc!{"id": u.tracking.id.clone()}, u, None)
                // .upsert(true)
                .await?;
        }

        todo!()
    }
}

#[get("/trip/<trip_id>")]
pub async fn get_trip(db: Connection<BrussData>, trip_id: TripId) -> ApiResponse<Vec<TripTracking>> {
    let db = DBInterface(db);
    let result = Queriable::<QueryResult<Trip>>::query(&db, trip_id.into()).await?.data;
    let mut out = vec![];
    for t in result.into_iter() {
        let ty = match Queriable::<Option<Route>>::query(&db, doc!{"id": t.route as i32}.into()).await? {
            Some(r) => r.area_ty,
            None => return ApiResponse::Error(ApiError::NotFound.into()),
        };
        out.push(TripTracking::from((ty, t)));
    }
    let tot = Some(out.len());
    ApiResponse::Ok(out.into_iter().map(TripTracking::from).collect(), tot)
}


lazy_static! {
    pub static ref ROUTES: Vec<rocket::Route> = routes![get_trip];
}
