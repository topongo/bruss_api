use chrono::{NaiveTime, Timelike};
use rocket::form::FromForm;
use tt::AreaType;
use mongodb::bson::{doc, Document};

#[derive(FromForm)]
pub struct TripQuery {
    id: Option<String>,
    time: Option<String>,
}

impl TripQuery {
    fn to_doc_time_stop(time: String, stop: u16) -> Document {
        let t = NaiveTime::parse_from_str(&time, "%H:%M").unwrap();
        doc!{
            "$expr": {
                "$and": [
                    {"$gte": [{"$hour": {"$toDate": { "$concat": ["1970-01-01T", format!("$times.{}.departure", stop), "Z"]}}}, t.hour() as i32]},
                    {"$gte": [{"$minute": {"$toDate": { "$concat": ["1970-01-01T", format!("$times.{}.departure", stop), "Z"]}}}, t.minute() as i32]},
                ]
            }
        }
    }

    fn to_doc_time_route(time: String) -> Document {
        let t = NaiveTime::parse_from_str(&time, "%H:%M").unwrap();
        doc!{
            "$expr": {
                "$anyElementTrue": {
                    "$map": {
                        "input": { "$objectToArray": "$times" },
                        "in": {
                            "$gt": [{ "$hour": { "$toDate": { "$concat": ["1970-01-01T", "$$this.v.departure", "Z"] } } }, t.hour() as i32],
                            "$gt": [{ "$minute": { "$toDate": { "$concat": ["1970-01-01T", "$$this.v.departure", "Z"] } } }, t.minute() as i32],
                        }
                    }
                }
            }
        }
    }

    pub fn to_doc_stop(self, stop: u16, ty: AreaType) -> Document {
        let Self { id, time } = self;
        let ty_st: &str = ty.into();
        let mut conds = vec![doc!{"type": ty_st}, doc!{format!("times.{}", stop): doc!{"$exists": true}}];
        if let Some(time) = time {
            conds.push(Self::to_doc_time_stop(time, stop));
        }
        if let Some(id) = id { conds.push(doc!{"id": id.clone()}) }
        let d = doc!{"$and": conds};
        info!("{:?}", d);
        d
    }

    pub fn to_doc_route(self, route: u16) -> Document {
        let Self { id, time } = self;
        let mut conds = vec![doc!{"route": route as i32}];
        if let Some(time) = time {
            conds.push(Self::to_doc_time_route(time));
        }
        if let Some(id) = id { conds.push(doc!{"id": id.clone()}) }
        let d = doc!{"$and": conds};
        info!("{:?}", d);
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

