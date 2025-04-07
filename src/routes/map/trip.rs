use bruss_data::{Direction, Trip};
use chrono::{DateTime, Local, NaiveTime, TimeDelta, Utc};
use lazy_static::lazy_static;
use rocket::form::FromForm;
use rocket::time::Time;
use serde::{Deserialize, Serialize};
use tt::AreaType;
use mongodb::bson::{doc, Document};
use super::query::DBQuery;
use super::pipeline::{CustomPipeline, Pipeline};

use super::{gen_generic_getters, FromStringFormField};

#[derive(FromForm)]
pub struct MultiTripQuery {
    time: Option<Time>,
    direction: Option<FromStringFormField<Direction>>,
}

#[derive(FromForm, Debug)]
pub struct TripQuerySingle {
    id: String,
}

impl DBQuery for TripQuerySingle {
    fn to_doc(self) -> Document {
        let Self { id } = self;        
        doc!{"id": id}
    }
}

#[derive(Deserialize, Serialize)]
pub struct TripCross {
    trip: Trip,
    #[serde(deserialize_with = "bson::serde_helpers::deserialize_chrono_datetime_from_bson_datetime")]
    departure: DateTime<Utc>,
    arrival_at_stop: Option<DateTimeUtcWrapper>,
}

#[derive(Serialize, Deserialize)]
struct DateTimeUtcWrapper(#[serde(deserialize_with = "bson::serde_helpers::deserialize_chrono_datetime_from_bson_datetime")] DateTime<Utc>);

fn rocket_time_to_chrono_utc(time: Time) -> chrono::DateTime<Utc> {
    DateTime::<Utc>::from(
        Local::now().with_time(
            NaiveTime::from_hms_opt(
                time.hour().into(), 
                time.minute().into(),
                time.second().into()
            ).unwrap()
        ).unwrap()
    )
}

impl MultiTripQuery {
    pub fn into_pipeline_route(self, route: u16, skip: Option<u32>, limit: Option<u32>) -> CustomPipeline {
        let Self { time, direction } = self;
        let time = match time {
            Some(t) => rocket_time_to_chrono_utc(t),
            None => Utc::now(),
        };

        let mut conds = vec![doc!{"hints.route": route as i32}];
        conds.push(doc!{"$expr": {"$lte": [
                // check that the current time is lower than...
                time, 
                // the general arrival time of the trip
                "$arrival",
        ]}});
        if let Some(direction) = direction {
            conds.push(doc!{"hints.direction": direction.into_bson()});
        }

        let limit = limit.map(|v| v as i64).unwrap_or_else(Pipeline::default_limit);
        let skip = skip.map(|v| v as i64).unwrap_or(0);

        let match_stage = doc!{"$match": {"$and": conds}};
        let sort_stage = doc!{"$sort": {"departure": 1}};
        let lookup_stage = doc!{"$lookup": {"from": "trips","localField": "id","foreignField": "id","as": "trip"}};
        let unwind_stage = doc!{"$unwind": "$trip"};
        let skip_stage = doc!{"$skip": skip};
        let limit_stage = doc!{"$limit": limit};
        let project_stage = doc!{"$project": {"_id": 0,"trip": 1,"departure": 1}};
        let count_stage = doc!{"$count": "count"};

        // for counting we only need the match and the count stage:
        // - sorting isn't needed
        // - lookup isn't needed since we don't do any filtering on the looked up data
        // - unwind, skip, limit and project aren't needed as the lookup stage is not performed
        let count = vec![
            match_stage.clone(),
            count_stage,
        ];
        
        // include all stages except for the count stage
        let fetch = vec![
            match_stage,
            sort_stage,
            lookup_stage,
            unwind_stage,
            skip_stage,
            limit_stage,
            project_stage
        ];

        Pipeline::custom(fetch, count)
    }

    pub fn into_pipeline_stop(self, stop: u16, area_type: AreaType, skip: Option<u32>, limit: Option<u32>) -> CustomPipeline {
        let Self { time, direction } = self;

        let time = match time {
            Some(t) => rocket_time_to_chrono_utc(t),
            None => Utc::now(),
        };

        let stop_time_string = format!("$hints.times.{}.departure", stop);
        let slen = stop_time_string.len();

        let mut conds = vec![
            doc!{"hints.type": area_type.to_string()},
            doc!{&stop_time_string[1..slen]: {"$exists": true}},
        ];
        if let Some(direction) = direction {
            conds.push(doc!{"hints.direction": direction.into_bson()});
        }

        let skip = skip.map(|v| v as i64).unwrap_or(0);
        let limit = limit.map(|v| v as i64).unwrap_or_else(Pipeline::default_limit);

        // we first filter out the great majority of trips using static conditions
        let match_stage = doc!{"$match": {"$and": conds}};
        let heuristic_match_stage = doc!{"$match": {"$and": [{"departure": {"$gte": time - TimeDelta::hours(4)}}, {"departure": {"$lt": time + TimeDelta::days(1)}}]}};
        // we then calculate the arrival time at the stop
        let set_stage = doc!{"$addFields": {"arrival_at_stop": {"$add": ["$departure", {"$multiply": [1000, {"$arrayElemAt": [&stop_time_string, 0]}]}]}}};
        // then we match based on the arrival time at the stop
        // we use a 20 minutes buffer to account for delays
        let match_arrival_stage = doc!{"$match": {"arrival_at_stop": {"$gte": time - TimeDelta::minutes(20)}}};
        // we sort by the arrival time at the stop
        let sort_stage = doc!{"$sort": {"arrival_at_stop": 1}};
        let lookup_stage = doc!{"$lookup": {"from": "trips","localField": "id","foreignField": "id","as": "trip"}};
        let unwind_stage = doc!{"$unwind": "$trip"};
        let skip_stage = doc!{"$skip": skip};
        let limit_stage = doc!{"$limit": limit};
        let project_stage = doc!{"$project": {"_id": 0,"trip": 1,"departure": 1, "arrival_at_stop": 1}};
        let count_stage = doc!{"$count": "count"};

        let count = vec![
            match_stage.clone(),
            heuristic_match_stage.clone(),
            set_stage.clone(),
            match_arrival_stage.clone(),
            count_stage,
        ];

        let fetch = vec![
            match_stage,
            heuristic_match_stage,
            set_stage,
            match_arrival_stage,
            sort_stage,
            lookup_stage,
            unwind_stage,
            skip_stage,
            limit_stage,
            project_stage
        ];

        Pipeline::custom(fetch, count)
    }
    
    // pub fn to_doc_route(self, route: u16) -> Document {
    //     let Self { id } = self;
    //     let mut d = Document::new();
    //     d.insert("route", route as i32);
    //     if let Some(id) = id { d.insert("id", id.clone()); }
    //     d 
    // }
}


gen_generic_getters!(Trip, TripQuerySingle, String);

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get, get_opts];
}
