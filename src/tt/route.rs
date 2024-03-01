use serde::{Serialize,Deserialize,Deserializer};

#[derive(Serialize,Deserialize,Debug)]
pub struct TTRoute {
    #[serde(alias="routeId")]
    id: u16,
    #[serde(alias="areaId")]
    area: u16,
    #[serde(deserialize_with="parse_color",alias="routeColor")]
    color: String,
    #[serde(alias="routeLongName")]
    name: String,
    #[serde(alias="routeShortName")]
    code: String,
    #[serde(alias="routeType")]
    ty: u16
}

fn parse_color<'de, D>(d: D) -> Result<String, D::Error> where D: Deserializer<'de> {
    Deserialize::deserialize(d)
        .map(|x: Option<_>| {
            x.unwrap_or("CCCCCC".to_string())
        })
}

