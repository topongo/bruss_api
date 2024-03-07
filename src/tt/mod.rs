mod area;
mod client;
mod error;
mod route;
mod stop;

pub use area::TTArea;
pub use client::TTClient;
pub use error::TTError;


pub type TTResult<T> = Result<T, TTError>;
