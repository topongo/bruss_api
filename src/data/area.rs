use serde::{Deserialize, Serialize};
use super::AreaType;

#[derive(Deserialize, Serialize, Debug)]
pub struct Area {
    pub id: u16,
    pub label: String,
    pub ty: AreaType
}

impl Area {
    pub(crate) fn new(id: u16, label: String, ty: AreaType) -> Self {
        Self { id, label, ty }
    }
}
