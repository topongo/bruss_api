use serde::{Serialize,Deserialize};
use tt::{TTStop,AreaType};

use super::BrussType;
use super::FromTT;
use super::Position;

#[derive(Serialize, Deserialize, Debug)]
pub struct Stop {
    id: u16,
    code: String,
    description: String,
    position: Position,
    altitude: i32,
    name: String,
    street: Option<String>,
    town: Option<String>,
    #[serde(rename = "type")]
    ty: AreaType,
    wheelchair_boarding: bool
}

impl Stop {
    pub fn new(id: u16, code: String, description: String, position: Position, altitude: i32, name: String, street: Option<String>, town: Option<String>, ty: AreaType, wheelchair_boarding: bool) -> Self {
        Self { id, code, description, position, altitude, name, street, town, ty, wheelchair_boarding }
    }
}

impl BrussType for Stop {
    const DB_NAME: &'static str = "stops";
}

impl FromTT<TTStop> for Stop {
    fn from_tt(value: TTStop) -> Self {
        let TTStop { id, code, description, lat, lng, altitude, name, street, town, ty, wheelchair_boarding } = value;
        Self { id, code, description, position: Position::new(lat, lng), altitude, name, street, town, ty, wheelchair_boarding }
    }
}

