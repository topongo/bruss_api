use serde::{Serialize,Deserialize};

use super::AreaType;
use super::BrussType;
use super::Position;

#[derive(Serialize, Deserialize, Debug)]
pub struct Stop {
    id: u16,
    code: String,
    description: String,
    position: Position,
    altitude: i32,
    name: String,
    street: String,
    town: String,
    #[serde(rename = "type")]
    ty: AreaType,
    wheelchair_boarding: bool
}

impl Stop {
    pub fn new(id: u16, code: String, description: String, position: Position, altitude: i32, name: String, street: String, town: String, ty: AreaType, wheelchair_boarding: bool) -> Self {
        Self { id, code, description, position, altitude, name, street, town, ty, wheelchair_boarding }
    }
}

impl BrussType for Stop {
    const DB_NAME: &'static str = "stops";
}
