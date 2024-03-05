use serde::{Serialize,Deserialize};

use super::BrussType;

#[derive(Serialize, Deserialize, Debug)]
pub struct Route {
    id: u16,
    area: u16,
    color: String,
    name: String,
    code: String,
    #[serde(rename(serialize = "type"))]
    ty: u16
}

impl Route {
    pub fn new(id: u16, area: u16, color: String, name: String, code: String, ty: u16) -> Self {
        Self { id, area, color, name, code, ty }
    }
}

impl BrussType for Route {
    const DB_NAME: &'static str = "routes";
}
