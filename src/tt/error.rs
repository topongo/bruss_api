use std::error::Error;
use std::fmt::{Debug, Display};

use reqwest::Url;

#[derive(Debug)]
pub enum TTError {
    HttpError(String, Option<String>),
    JsonError(String, Option<String>),
}

impl From<reqwest::Error> for TTError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_decode() {
            TTError::JsonError(value.to_string(), value.url().map(Url::to_string))
        } else {
            TTError::HttpError(value.to_string(), value.url().map(Url::to_string))
        }
    }
}

impl Error for TTError {}

impl Display for TTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

