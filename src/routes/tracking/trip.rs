use std::collections::{HashMap, HashSet};
use std::time::Duration;

use bruss_config::CONFIGS;
use bruss_data::{BrussType, Route, Trip};
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use mongodb::options::ReplaceOptions;
use rocket::request::FromParam;
use rocket_db_pools::Connection;
use serde::{Serialize,Deserialize};
use mongodb::bson::{doc, Document};
use tt::{AreaType, ParallelRequester, TTTrip};
use crate::{db::BrussData, response::ApiResponse, routes::map::query::{DBInterface, DBQuery}};

#[derive(Debug, Serialize, Deserialize)]
pub struct TripTracking {
    id: String,
    delay: i32,
    last_stop: Option<u16>,
    next_stop: Option<u16>,
    area: Option<AreaType>,
    bus_id: Option<u16>,
    last_event: Option<DateTime<Utc>>,
}

impl TripTracking {
    #[allow(dead_code)]
    fn error(id: String) -> Self {
        Self {
            id,
            delay: 0,
            last_stop: None,
            next_stop: None,
            area: None,
            bus_id: None,
            last_event: None,
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
            last_event: value.last_event,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TripIds(Vec<String>);

impl FromParam<'_> for TripIds {
    type Error = ();

    fn from_param(param: &'_ str) -> Result<Self, Self::Error> {
        Ok(TripIds(param.split(',').map(str::to_string).collect::<Vec<String>>()))
    }

}

impl DBQuery for TripIds {
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
    async fn get_by_ids(db: DBInterface, id: Vec<String>) -> Result<Vec<Self>, mongodb::error::Error> {
        let now = Utc::now();
        // sanitize id vec:
        let id = id.into_iter().collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();

        let coll = db.0.database(CONFIGS.db.get_db())
            .collection::<TripUpdate>("trip_updates");
        let cached: HashMap<String, TripUpdate> = coll
            .find(doc!{"id": doc!{"$in": &id}, "updated": doc!{"$gt": (now - Duration::from_secs(CONFIGS.api.max_rt_age)).timestamp()}}, None)
            .await?
            .map(|r| r.map(|d| (d.tracking.id.clone(), d)))
            .try_collect()
            .await?;

        let cli = CONFIGS.tt.client();
        let id_len = id.len();
        let p_requester = ParallelRequester::<TTTrip>::new(cli, CONFIGS.routing.parallel_downloads.unwrap_or(1));
        for i in id.into_iter()
            .filter(|i| !cached.contains_key(i))
        {
            p_requester.request_one(i).await
        }


        let tt_updates = p_requester.gather().await
            .map_err(mongodb::error::Error::custom)?
            .into_iter()
            .map(Trip::from_tt)
            .map(|v| v.0)
            .collect::<Vec<Trip>>();

        let routes = tt_updates.iter()
            .map(|t| t.route)
            .collect::<HashSet<u16>>();

        // get needed routes (one usually) from db
        let areas: HashMap<u16, AreaType> = Route::get_coll(&db.0.database(CONFIGS.db.get_db()))
            .find(doc!{"id": doc!{"$in": routes.iter().map(|u| *u as i32).collect::<Vec<i32>>()}}, None)
            .await?
            .map(|r| r.map(|r| (r.id, r.area_ty)))
            .try_collect()
            .await?;

        let db_updates = tt_updates.into_iter()
            .map(|t: Trip| TripTracking::from((areas[&t.route], t)))
            .map(|t| TripUpdate { tracking: t, updated: now })
            .collect::<Vec<_>>();
        
        let r = ReplaceOptions::builder().upsert(true).build();
        for u in db_updates.iter() {
            coll
                .replace_one(doc!{"id": u.tracking.id.clone()}, u, Some(r.clone()))
                .await?;
        }

        let mut output = db_updates;
        output.extend(cached.into_values());
        debug_assert_eq!(output.len(), id_len);
        Ok(output)
    }
}

impl From<TripUpdate> for TripTracking {
    fn from(value: TripUpdate) -> Self {
        value.tracking
    }
}

#[get("/trip/<trip_ids>")]
pub async fn get_trip(db: Connection<BrussData>, trip_ids: TripIds) -> ApiResponse<Vec<TripTracking>> {
    let db = DBInterface(db);

    let trips = TripUpdate::get_by_ids(db, trip_ids.0).await?;
    let tot = trips.len();
    
    ApiResponse::Ok(trips.into_iter().map(|t| t.into()).collect(), Some(tot))
}


lazy_static! {
    pub static ref ROUTES: Vec<rocket::Route> = routes![get_trip];
}
