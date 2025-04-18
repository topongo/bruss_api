use serde::Serialize;
use rocket::request::FromParam;
use super::FromStringFormField;
use tt::AreaType;
use super::query::DBQuery;
use std::{num::ParseIntError, str::FromStr};
use mongodb::bson::Document;

#[derive(Debug)]
pub struct Id<T: Serialize> {
    inner: T
}

#[derive(Debug)]
pub struct ParamError<T: std::error::Error>(T);

impl<T: std::error::Error> From<T> for ParamError<T> {
    fn from(value: T) -> Self {
        ParamError(value)
    }
}

impl<'a> FromParam<'a> for Id<u16> {
    type Error = ParamError<ParseIntError>;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(Id { inner: param.parse::<u16>()? })
    }
}

impl<'a, T> FromParam<'a> for Id<FromStringFormField<T>> where T: FromStr + Serialize, T::Err: std::error::Error {
    type Error = ParamError<T::Err>;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        param.parse()
            .map(|v| Id { inner: FromStringFormField { inner: v } })
            .map_err(|e| ParamError(e))
    }
}

impl<'a> FromParam<'a> for Id<String> {
    type Error = ParamError<ParseIntError>;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(Id { inner: param.to_owned() })
    }
}

/// `ParamQuery`: A trait for converting a generic parameter into a mongodb document query.
pub trait ParamQuery<T>: DBQuery {
    fn key(&self) -> &'static str;
    fn value(self) -> T;
}

impl ParamQuery<i32> for Id<u16> {
    fn key(&self) -> &'static str {
        "id"
    }

    fn value(self) -> i32 {
        self.inner as i32
    }
}

impl DBQuery for Id<u16> {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        d.insert(self.key(), self.value());
        d
    }
}

impl ParamQuery<FromStringFormField<AreaType>> for Id<FromStringFormField<AreaType>> {
    fn key(&self) -> &'static str {
        "type"
    }

    fn value(self) -> FromStringFormField<AreaType> {
        self.inner
    }
}

impl DBQuery for Id<FromStringFormField<AreaType>> {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        d.insert::<_,  &'static str>(self.key(), self.value().inner.into());
        d
    }
}

impl ParamQuery<String> for Id<String> {
    fn key(&self) -> &'static str {
        "id"
    }

    fn value(self) -> String {
        self.inner
    }
}

impl DBQuery for Id<String> {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        d.insert(self.key(), self.value());
        d
    }
}

