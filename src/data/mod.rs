mod area;
mod area_type;
mod route;
mod position;
mod stop;

pub use area::Area;
pub use area_type::AreaType;
pub use route::Route;
pub use stop::Stop;
pub use position::Position;

use serde::{de::DeserializeOwned, Serialize};

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

