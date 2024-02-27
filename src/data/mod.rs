use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AreaType {
    E,
    U
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Area {
    pub id: u16,
    pub label: String,
    pub ty: AreaType
}

