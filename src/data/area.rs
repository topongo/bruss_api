use serde::{Deserialize, Serialize};
use super::{AreaType, BrussType};

#[derive(Deserialize, Serialize, Debug)]
pub struct Area {
    pub id: u16,
    pub label: String,
    #[serde(rename = "type")]
    pub ty: AreaType
}

impl Area {
    pub(crate) fn new(id: u16, label: String, ty: AreaType) -> Self {
        Self { id, label, ty }
    }
}

impl BrussType for Area {
    const DB_NAME: &'static str = "areas";
}
