use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AreaType {
    #[serde(rename(serialize = "e"), alias = "e")]
    E,
    #[serde(rename(serialize = "u"), alias = "u")]
    U
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Area {
    pub id: u16,
    pub label: String,
    pub ty: AreaType
}

