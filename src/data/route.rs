use serde::{Serialize,Deserialize};
use tt::TTRoute;

use super::{BrussType, FromTT};

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

impl FromTT<TTRoute> for Route {
    fn from_tt(value: TTRoute) -> Self {
        let TTRoute { id, ty, area, color, name, code } = value;
        Self { id, ty, area, color, name, code }
    }
}

