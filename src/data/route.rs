use serde::{Serialize,Deserialize};

use super::BrussType;

#[derive(Serialize, Deserialize, Debug)]
pub struct Route {
    id: u16,
    #[serde(rename = "type")]
    ty: u16,
    area: u16,
    color: String,
    name: String,
    code: String,
}

impl Route {
    pub fn new(id: u16, ty: u16, area: u16, color: String, name: String, code: String) -> Self {
        Self { id, area, color, name, code, ty }
    }
}

impl BrussType for Route {
    const DB_NAME: &'static str = "routes";
}
