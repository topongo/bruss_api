use serde::{Serialize,Deserialize,Deserializer};

use crate::data::{AreaType,Stop,ToBruss,Position};

#[derive(Serialize,Deserialize,Debug)]
pub struct TTStop {
    #[serde(alias = "stopId")]
    id: u16,
    #[serde(alias = "stopCode")]
    code: String,
    #[serde(alias = "stopDesc")]
    description: String,
    #[serde(alias = "stopLat")]
    lat: f32,
    #[serde(alias = "stopLon")]
    lng: f32,
    #[serde(alias = "stopLevel")]
    altitude: i32,
    #[serde(alias = "stopName")]
    name: String,
    street: String,
    town: String,
    #[serde(rename = "type")]
    ty: AreaType,
    #[serde(alias = "wheelchairBoarding")]
    wheelchair_boarding: bool
}

impl ToBruss for TTStop {
    type Output = Stop;

    fn to_bruss(self) -> Self::Output {
        let Self { id, code, description, lat, lng, altitude, name, street, town, ty, wheelchair_boarding } = self;
        Self::Output::new(id, code, description, Position::new(lat, lng), altitude, name, street, town, ty, wheelchair_boarding)
    }
}


