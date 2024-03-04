use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AreaType {
    #[serde(rename(serialize = "e"), alias = "e")]
    E,
    #[serde(rename(serialize = "u"), alias = "u")]
    U
}
