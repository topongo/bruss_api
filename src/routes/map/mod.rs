mod route;
mod area;
mod db;
mod segment;
mod stop;
mod trip;
mod path;

pub use area::get_areas;
pub use route::get_routes;
pub use segment::{get_segments,get_segments_poly};
use serde::Deserialize;
pub use stop::get_stops;
pub use trip::{get_trips_route,get_trips_stop};
pub use path::get_path;

use tt::AreaType;
use rocket::form::FromFormField;
use rocket::form::Error;

#[derive(Deserialize)]
#[serde(transparent)]
struct AreaTypeWrapper {
    inner: AreaType
}

impl FromFormField<'_> for AreaTypeWrapper {
    fn from_value(field: rocket::form::ValueField<'_>) -> rocket::form::Result<'_,Self> {
        match field.value {
            "u" => Ok(AreaTypeWrapper { inner: AreaType::U }),
            "e" => Ok(AreaTypeWrapper { inner: AreaType::E }),
            _ => Err(Error::validation("could be either \"u\" or \"e\"").into())
        }
    }
}
