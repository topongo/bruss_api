use std::convert::Infallible;
use std::ops::FromResidual;

use futures::TryStreamExt;
use rocket::serde::json::Json;
use rocket::http::Status;
use mongodb::bson::Document;
use rocket_db_pools::Connection;
use bruss_config::CONFIGS;
use bruss_data::BrussType;
use crate::db::BrussData;
use mongodb::error::{Error as MongoError, ErrorKind};
use serde::Serialize;

#[derive(Serialize)]
struct DBError {
    // kind: ErrorKind,
    message: String
}

impl From<MongoError> for DBError {
    fn from(value: MongoError) -> Self {
        DBError { message: value.to_string(), /* kind: value.kind */ }
    }
}

#[derive(Responder)]
pub enum DBResponse<T> {
    #[response(status = 200)]
    Ok { inner: (Status, Json<T>) },
    // #[response(status = 500, content_type = "json")]
    // SerializationError(String),
    #[response(status = 500, content_type = "json")]
    DBError {
        inner: Json<DBError>
    }
}

impl<T> From<Result<T, mongodb::error::Error>> for DBResponse<T> {
    fn from(value: Result<T, mongodb::error::Error>) -> Self {
        match value {
            Ok(v) => DBResponse::Ok {
                inner: (Status::Ok, Json(v)),
            },
            Err(e) => {
                error!("Internal Server Error: {:?}", e);
                DBResponse::DBError { inner: Json(e.into()) }
            },
        }
    }
} 

impl<T> FromResidual<Result<Infallible, mongodb::error::Error>> for DBResponse<T> {
    fn from_residual(residual: Result<Infallible, mongodb::error::Error>) -> Self {
        match residual {
            Ok(_inf) => panic!(),
            Err(err) => DBResponse::DBError { inner: Json(err.into()) }
        }
    }
}

pub trait DBQuery {
    fn to_doc(&self) -> Document;
}

pub async fn db_query_get<T, Q>(db: Connection<BrussData>, query: Q) -> DBResponse<Vec<T>>
    where T: BrussType, Q: DBQuery 
{
    db_query_doc(db, query.to_doc()).await
}

pub async fn db_query_json<T, Q>(db: Connection<BrussData>, query: Json<Q>) -> DBResponse<Vec<T>>
    where T: BrussType, Q: DBQuery
{
    db_query_doc(db, query.to_doc()).await
}

pub async fn db_query_doc<T>(db: Connection<BrussData>, query: Document) -> DBResponse<Vec<T>>
    where T: BrussType
{
    match T::get_coll(&db.database(CONFIGS.db.get_db()))
        .find(query, None)
        .await
    {
        Ok(found) => found.try_collect::<Vec<T>>().await.into(),
        Err(e) => DBResponse::DBError { inner: Json(e.into()) }
    }
}


