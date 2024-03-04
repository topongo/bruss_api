mod area;
mod client;
mod error;
mod route;

pub use area::TTArea;
pub use client::TTClient;
pub use error::TTError;

/// Convertible to bruss data, that will be accepted by the database
pub trait ToBruss {
    type Output;

    fn to_bruss(self) -> Self::Output;
}

pub type TTResult<T> = Result<T, TTError>;
