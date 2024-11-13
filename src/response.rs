use std::convert::Infallible;
use std::ops::FromResidual;

use rocket::form::Errors;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::Response;
use rocket::{serde::json::Json, Request};
use serde::Serialize;
use serde::ser::SerializeMap;

use crate::routes::map::params::ParamError;
use crate::routes::map::query::QueryResult;

pub enum ApiResponse<T> {
    Ok(T, Option<usize>),
    Error(ApiError)
}

impl<T> ApiResponse<T> {
    pub fn status(&self) -> u16 {
        match self {
            Self::Ok(..) => 200,
            Self::Error(e) => e.status()
        }
    }
}

impl<T: Serialize> Serialize for ApiResponse<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {

        match self {
            Self::Ok(v, ..) => v.serialize(serializer),
            Self::Error(e) => e.serialize(serializer)
        }
    }
}

impl<'r, 'o: 'r, T: Serialize> Responder<'r, 'o> for ApiResponse<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        // if let Self::Error(e) = self {
        //     if matches!(e, ApiError::NotFound(_)) {
        //         Err(Status::from_code(404).unwrap())
        //     } else {
        //         Response::build()
        //             .status(Status::from_code(e.status()).unwrap())
        //             .merge(Json::respond_to(Json(e), request)?)
        //             .ok()
        //    }
        // } else {
        let mut build = Response::build();
        build.status(Status::new(self.status()));
        if let Self::Ok(_, Some(count)) = self {
            build.raw_header("X-Total-Count", count.to_string());
        }
        build
            .merge(Json(self).respond_to(request)?)
            .ok()
        // }
    }

    
}

impl<T> From<Result<QueryResult<T>, mongodb::error::Error>> for ApiResponse<Vec<T>> {
    fn from(value: Result<QueryResult<T>, mongodb::error::Error>) -> Self {
        match value {
            Ok(v) => ApiResponse::Ok(v.data, Some(v.total)),
            Err(e) => ApiResponse::Error(ApiError::InternalServer(Box::new(e)))
        }
    }
}

impl<T> From<Result<Option<T>, mongodb::error::Error>> for ApiResponse<T> {
    fn from(value: Result<Option<T>, mongodb::error::Error>) -> Self {
        match value {
            Ok(v) => match v {
                Some(v) => ApiResponse::Ok(v, None),
                None => ApiError::NotFound.respond() 
            }
            Err(e) => ApiError::InternalServer(Box::new(e)).respond()
        }
    }
}

impl<T> FromResidual<Result<Infallible, mongodb::error::Error>> for ApiResponse<T> {
    fn from_residual(residual: Result<Infallible, mongodb::error::Error>) -> Self {
        match residual {
            Ok(_inf) => panic!(),
            Err(e) => ApiError::InternalServer(Box::new(e)).respond()
        }
    }
}

impl<T> FromResidual<Option<Infallible>> for ApiResponse<T> {
    fn from_residual(residual: Option<Infallible>) -> Self {
        match residual {
            Some(_inf) => panic!(),
            None => ApiResponse::Error(ApiError::NotFound)
        }
    }
}

impl<T> FromResidual<Result<Infallible, Errors<'_>>> for ApiResponse<T> {
    fn from_residual(residual: Result<Infallible, Errors<'_>>) -> Self {
        match residual {
            Ok(_inf) => panic!(),
            Err(e) => {
                ApiError::Form(e.into_iter().map(|e| e.into()).collect()).respond()
            } 
        }
    }
}

impl<T, E: std::error::Error> FromResidual<Result<Infallible, ParamError<E>>> for ApiResponse<T> {
    fn from_residual(residual: Result<Infallible, ParamError<E>>) -> Self {
        panic!();
        match residual {
            Ok(_inf) => panic!(),
            Err(_e) => ApiError::Form(vec![FormError { name: None, value: None, kind: "invalid identifier".to_owned() }]).respond()
        }
    }
}

impl<T> FromResidual<Result<Infallible, tt::TTError>> for ApiResponse<T> {
    fn from_residual(residual: Result<Infallible, tt::TTError>) -> Self {
        match residual {
            Ok(_inf) => panic!(),
            Err(e) => ApiError::Generic(500, e.to_string()).respond()
        }
    }
}

#[derive(Serialize)]
pub struct FormError {
    name: Option<String>,
    value: Option<String>,
    kind: String,
    // entity: String, 
}

impl From<rocket::form::Error<'_>> for FormError {
    fn from(value: rocket::form::Error) -> Self {
        FormError { 
            name: value.name.map(|v| v.to_string()),
            value: value.value.map(|v| v.to_string()),
            kind: value.kind.to_string(),
            // entity: value.entity.to_string()
        }
    }
}

pub enum ApiError {
    NotFound,
    InternalServer(Box<dyn std::error::Error>),
    Generic(u16, String),
    Form(Vec<FormError>)
}

impl ApiError {
    pub fn status(&self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::InternalServer(_) => 500,
            Self::Generic(c, _) => *c,
            Self::Form(_) => 422,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::NotFound => "not found",
            Self::InternalServer(_) => "internal server error",
            Self::Generic(_, n) => n,
            Self::Form(_) => "unprocessable entity",
        }
    }

    pub fn respond<T>(self) -> ApiResponse<T> {
        ApiResponse::Error(self)
    }
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
        
        let mut map = serializer.serialize_map(None)?;
        // map.serialize_entry("status", &self.status())?;
        map.serialize_entry("error", &self.name())?;
        match self {
            // Self::NotFound(t) => {
            //     map.serialize_entry("uri", &uri)?;
            // },
            Self::InternalServer(e) => {
                #[cfg(debug_assertions)]
                map.serialize_entry("error_debug", &e.to_string())?;
            }
            Self::Form(e) => {
                map.serialize_entry("errors", &e)?;
            }
            _ => {}
        }
        map.end()
    }
}

#[catch(default)]
pub fn api_catch_default(status: Status, _req: &Request) -> Json<ApiResponse<()>> {
    Json(ApiError::Generic(status.code, "generic error".to_owned()).respond())
}

#[catch(404)]
pub fn api_catch_404(_req: &Request) -> Json<ApiResponse<()>> {
    Json(ApiError::NotFound.respond())
}
