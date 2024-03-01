use std::error::Error;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum TTError {
    HttpError(String),
    JsonError(String),
}

impl From<reqwest::Error> for TTError {
    fn from(value: reqwest::Error) -> Self {
        TTError::HttpError(value.to_string())
    }
}

impl Error for TTError {}

impl Display for TTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

