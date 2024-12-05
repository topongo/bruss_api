use bruss_data::Trip;
use chrono::{DateTime, Local, NaiveTime, Timelike, Utc};
use lazy_static::lazy_static;
use rocket::form::FromForm;
use rocket::time::Time;
use tt::AreaType;
use mongodb::bson::{doc, Document};
use super::query::DBQuery;
use super::pipeline::Pipeline;

use super::gen_generic_getters;

#[derive(FromForm)]
pub struct TripQuery {
    id: Option<Vec<String>>,
    time: Option<Time>,
}

// #[derive(FromForm,Debug)]
// pub struct TripQuerySingle {
//     id: Vec<String>,
// }

// impl DBQuery for TripQuerySingle {
//     fn to_doc(self) -> Document {
//         let Self { id } = self;
//         doc!{"id": doc!("$in": id)}
//     }
// }

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

impl TripQuery {
    fn to_doc_time_stop(t: NaiveTime, stop: u16) -> Document {
        doc!{
            "$expr": {
                "$or": [
                    { "$and": [
                        {"$gt": [{"$hour": {"$toDate": { "$concat": ["1970-01-01T", format!("$times.{}.departure", stop), "Z"]}}}, t.hour() as i32]},
                    ]},
                    { "$and": [
                        {"$eq": [{"$hour": {"$toDate": { "$concat": ["1970-01-01T", format!("$times.{}.departure", stop), "Z"]}}}, t.hour() as i32]},
                        {"$gte": [{"$minute": {"$toDate": { "$concat": ["1970-01-01T", format!("$times.{}.departure", stop), "Z"]}}}, t.minute() as i32]},
                    ]}
                ]
            }
        }
    }

    fn to_doc_time_route(t: NaiveTime) -> Document {
        doc!{
            "$expr": {
                "$anyElementTrue": {
                    "$map": {
                        "input": { "$objectToArray": "$times" },
                        "in": {
                            "$or": [
                                { "$and": [
                                    {"$gt": [{ "$hour": { "$toDate": { "$concat": ["1970-01-01T", "$$this.v.departure", "Z"] } } }, t.hour() as i32]},
                                ]},
                                { "$and": [
                                    {"$eq": [{ "$hour": { "$toDate": { "$concat": ["1970-01-01T", "$$this.v.departure", "Z"] } } }, t.hour() as i32]},
                                    {"$gte": [{ "$minute": { "$toDate": { "$concat": ["1970-01-01T", "$$this.v.departure", "Z"] } } }, t.minute() as i32]},
                                ]}
                            ]
                        }
                    }
                }
            }
        }
    }

    fn _rocket_time_to_chrono_utc(time: Time) -> NaiveTime {
        DateTime::<Utc>::from(
            Local::now().with_time(
                NaiveTime::from_hms_opt(
                    time.hour().into(), 
                    time.minute().into(),
                    time.second().into()
                ).unwrap()
            ).unwrap()
        ).time()
    }

    pub fn to_doc_stop(self, stop: u16, ty: AreaType) -> Document {
        let Self { id, time } = self;
        let ty_st: &str = ty.into();
        let mut conds = vec![doc!{"type": ty_st}, doc!{format!("times.{}", stop): doc!{"$exists": true}}];
        if let Some(time) = time {
            conds.push(Self::to_doc_time_stop(Self::_rocket_time_to_chrono_utc(time), stop));
        }
        if let Some(id) = id { conds.push(doc!{"id": id.clone()}) }
        let d = doc!{"$and": conds};
        d
    }

    pub fn to_doc_route(self, route: u16) -> Document {
        let Self { id, time } = self;
        let mut conds = vec![doc!{"route": route as i32}];
        if let Some(time) = time {
            println!("requested time: {:?}", time);
            let u = Self::_rocket_time_to_chrono_utc(time);
            println!("converted time: {:?}", u);
            conds.push(Self::to_doc_time_route(u));
        }
        if let Some(id) = id { conds.push(doc!{"id": id.clone()}) }
        let d = doc!{"$and": conds};
        d
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
