use serde::Deserialize;

use crate::data::{Area, AreaType};

#[derive(Deserialize, Debug)]
pub struct TTArea {
    #[serde(rename = "areaId")]
    id: u16,
    #[serde(rename = "areaDesc")]
    desc: String,
    #[serde(rename = "type")]
    ty: AreaType,
}

impl TTArea {
    pub fn to_area(self) -> Area {
        Area { id: self.id, label: self.desc, ty: self.ty }
    }
}

