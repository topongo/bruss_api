use bruss_data::{Direction, Trip};
use chrono::{DateTime, Local, NaiveTime, Utc};
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
pub struct TripDeparture {
    trip: Trip,
    #[serde(deserialize_with = "bson::serde_helpers::deserialize_chrono_datetime_from_bson_datetime")]
    departure: DateTime<Utc>,
}


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
    // let q = vec![ 
    //     doc!{"$match": {"$and": [
    //         // filter by area
    //         {"hints.type": area.to_string()}, 
    //         // filter by route
    //         {"hints.route": route as i32}, 
    //         // filter by general departure
    //         {"departure": {"$gte": time}},
    //     ]}}, 
    //     // sort by general departure
    //     doc!{"$sort": {"departure": 1}},
    //     // lookup trip
    //     doc!{"$lookup": {"from": "trips","localField": "id","foreignField": "id","as": "trip"}},
    //     // strip $lookup result
    //     doc!{"$unwind": "$trip"}, 
    //     // hard limit results
    //     doc!{"$limit": 100},
    //     // project only the necessary fields
    //     doc!{"$project": {"_id": 0,"trip": 1,"departure": 1}},
    // ];
        let Self { time, .. } = self;
        let time = match time {
            Some(t) => rocket_time_to_chrono_utc(t),
            None => Utc::now(),
        };
        let match_stage = doc!{"$match": {"$and": [
            // filter by route
            {"hints.route": route as i32}, 
            // filter by general departure
            {"departure": {"$gte": time}},
        ]}};
        let sort_stage = doc!{"$sort": {"departure": 1}};
        let lookup_stage = doc!{"$lookup": {"from": "trips","localField": "id","foreignField": "id","as": "trip"}};
        let unwind_stage = doc!{"$unwind": "$trip"};
        let skip_stage = doc!{"$skip": skip.map(|v| v as i64).unwrap_or(0)};
        let limit_stage = doc!{"$limit": limit.map(|v| v as i64).unwrap_or_else(Pipeline::default_limit)};
        let project_stage = doc!{"$project": {"_id": 0,"trip": 1,"departure": 1}};
        // let project_stage = doc!{"$replaceRoot": {"newRoot": "$trip"}};
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
        // vec![
        //     doc!{"$match": {"$and": [
        //         // filter by area
        //         {"hints.type": area.to_string()}, 
        //         // filter by stop, if present in hits.times
        //         {&stop_time_string[1..slen - 3]: {"$exists": true}}, 
        //     ]}}, 
        //     // calculate arrival at stop
        //     doc!{"$set": {"arrival_at_stop": {"$add": ["$departure", {"$multiply": [1000, {"$arrayElemAt": [&stop_time_string[0..slen - 3], 0]}]}]}}},
        //     // filter by arrival at stop
        //     doc!{"$match": {"arrival_at_stop": {"$gte": time}}},
        //     // sort by arrival at specific stop: we can't simply sort by general departure.
        //     doc!{"$sort": {"arrival_at_stop": 1}},
        //     // lookup trip
        //     doc!{"$lookup": {"from": "trips","localField": "id","foreignField": "id","as": "trip"}},
        //     // strip $lookup result
        //     doc!{"$unwind": "$trip"}, 
        //     // hard limit results
        //     doc!{"$limit": 100},
        //     // project only the necessary fields
        //     doc!{"$project": {"_id": 0,"trip": 1,"departure": 1}},
        //     // doc!{"$set": {"arrival_at_stop": }}
        // ]
        let Self { time, .. } = self;

        todo!()
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
