use bruss_data::{PolySegment, Segment};
use lazy_static::lazy_static;
use rocket::request::FromParam;
use tt::AreaType;
use crate::db::BrussData;
use mongodb::bson::{Document,doc};
use rocket_db_pools::Connection;
use super::{params::{Id, ParamError, ParamQuery}, pipeline::Pipeline, query::{DBInterface, UniformQueryable, QueryResult}, FromStringFormField};
use serde::{Serialize,Deserialize};
use crate::response::ApiResponse;
use std::{error::Error as StdError, fmt::Display, num::ParseIntError};


#[derive(FromFormField,Deserialize,Clone,Default)]
enum FormatSelect {
    #[field(value = "poly")]
    Polyline,
    #[default]
    #[field(value = "coords")]
    Coords,
}

#[derive(Debug)]
struct FormatSelectParseError;

impl Display for FormatSelectParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for FormatSelectParseError {}

impl<'a> FromParam<'a> for FormatSelect {
    type Error = ParamError<FormatSelectParseError>;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match param {
            "poly" => Ok(Self::Polyline),
            "coords" => Ok(Self::Coords),
            _ => Err(ParamError::from(FormatSelectParseError))
        }
    }
}

struct StopPairs(Vec<(u16, u16)>);

impl StopPairs {
    fn to_doc(self, ty: FromStringFormField<AreaType>) -> Document {
        let mut d = Document::new();
        let Self(pairs) = self;

        if pairs.len() > 0 {
            d.insert("$or", pairs.iter()
                .map(|(s1, s2)| doc!{"from": *s1 as i32, "to": *s2 as i32})
                .collect::<Vec<Document>>()
            );
            d.insert::<&'static str, &'static str>("type", ty.inner.into());
        } else {
            #[cfg(not(debug_assertions))]
            // deny getting all segments at one in prod
            d.insert("type", "nope");
        }
        d
    }
}

#[derive(Debug)]
enum StopPairsParseError {
    InvalidPair,
    ParseInt
}

impl Display for StopPairsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for StopPairsParseError {}

impl From<ParseIntError> for StopPairsParseError {
    fn from(_value: ParseIntError) -> Self {
        Self::ParseInt
    }
}

impl<'a> FromParam<'a> for StopPairs {
    type Error = ParamError<StopPairsParseError>;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        let mut pairs = Vec::new();
        for p in param.split(",") {
            if p.len() == 0 {
                continue;
            }
            let ss = p.split("-").collect::<Vec<&str>>();
            if ss.len() != 2 {
                return Err(StopPairsParseError::InvalidPair.into())
            }
            let s0 = match ss[0].parse() {
                Ok(v) => v,
                Err(e) => return Err(<ParseIntError as Into<StopPairsParseError>>::into(e).into()),
            };
            let s1 = match ss[1].parse() {
                Ok(v) => v,
                Err(e) => return Err(<ParseIntError as Into<StopPairsParseError>>::into(e).into()),
            };
            pairs.push((s0, s1))
        }
        Ok(Self(pairs))
    }
}


#[get("/<area_type>/<pairs>?<format>")]
async fn get<'a>(
    db: Connection<BrussData>,
    area_type: Result<Id<FromStringFormField<AreaType>>, <Id<FromStringFormField<AreaType>> as FromParam<'_>>::Error>,
    pairs: Result<StopPairs, ParamError<StopPairsParseError>>,
    format: Option<FormatSelect>
) -> ApiResponse<SegmentFormatWrapper> {
    let fmt = format.unwrap_or_default();

    let pipeline= Pipeline::from(pairs?.to_doc(area_type?.value()));
    
    let w: SegmentFormatWrapper = (
        UniformQueryable::<Segment>::query(&DBInterface(db), pipeline.build()).await?,
        fmt
    ).into();
    w.into()
}

struct SegmentFormatWrapper(QueryResult<Segment>, FormatSelect);

impl Serialize for SegmentFormatWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let Self(inner, format) = self;
        match format {
            FormatSelect::Coords => inner.data.serialize(serializer),
            FormatSelect::Polyline => {
                let inner: Vec<PolySegment> = inner.data.iter()
                    .map(|s| s.clone().into())
                    .collect();
                inner.serialize(serializer)
            }
        }
    }
}

impl From<(QueryResult<Segment>, FormatSelect)> for SegmentFormatWrapper {
    fn from(value: (QueryResult<Segment>, FormatSelect)) -> Self {
        SegmentFormatWrapper(value.0, value.1)
    }
}

impl Into<ApiResponse<SegmentFormatWrapper>> for SegmentFormatWrapper {
    fn into(self) -> ApiResponse<SegmentFormatWrapper> {
        let c = self.0.data.len();
        ApiResponse::Ok(self, Some(c))
    }
}

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get];
}

