mod area;
mod area_type;
mod route;

pub use area::Area;
pub use area_type::AreaType;
pub use route::Route;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub(crate) trait BrussType: Serialize + DeserializeOwned {
    const DB_NAME: &'static str;
}

/// Struct that can be converted to a bruss-compatible data, that will be serialized inside a
/// database.
pub trait ToBruss {
    type Output: BrussType;
    
    /// Convert to a
    fn to_bruss(self) -> Self::Output;
}

