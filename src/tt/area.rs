use serde::Deserialize;

use crate::data::{Area, AreaType};

use super::ToBruss;

#[derive(Deserialize, Debug)]
pub struct TTArea {
    #[serde(rename = "areaId")]
    id: u16,
    #[serde(rename = "areaDesc")]
    label: String,
    #[serde(rename = "type")]
    ty: AreaType,
}

impl ToBruss for TTArea {
    type Output = Area;

    fn to_bruss(self) -> Self::Output {
        Area::new(self.id, self.label, self.ty)
    }
}

