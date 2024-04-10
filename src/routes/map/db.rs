use futures::TryStreamExt;
use rocket::serde::json::Json;
use rocket::http::Status;
use mongodb::bson::Document;
use rocket_db_pools::Connection;
use bruss_config::CONFIGS;
use bruss_data::BrussType;
use crate::db::BrussData;

#[derive(Responder)]
pub enum DBResponse<T> {
    #[response(status = 200)]
    Ok { inner: (Status, Json<T>) },
    // #[response(status = 500, content_type = "json")]
    // SerializationError(String),
    #[response(status = 500)]
    DBError(String),
}

impl<T> From<Result<T, mongodb::error::Error>> for DBResponse<T> {
    fn from(value: Result<T, mongodb::error::Error>) -> Self {
        match value {
            Ok(v) => DBResponse::Ok {
                inner: (Status::Ok, Json(v)),
            },
            Err(e) => {
                error!("Internal Server Error: {:?}", e);
                DBResponse::DBError("error fetching data".to_owned())
            },
        }
    }
} 

pub trait DBQuery {
    fn to_doc(&self) -> Document;
}

pub async fn db_query_get<T, Q>(db: Connection<BrussData>, query: Q) -> DBResponse<Vec<T>>
    where T: BrussType, Q: DBQuery 
{
    info!("using collection: {}", T::DB_NAME);
    match db
        .database(CONFIGS.db.get_db())
        .collection::<T>(T::DB_NAME)
        .find(query.to_doc(), None)
        .await
    {
        Ok(found) => found.try_collect::<Vec<T>>().await.into(),
        Err(e) => DBResponse::DBError(e.to_string())
    }
}

pub async fn db_query_json<T, Q>(db: Connection<BrussData>, query: Json<Q>) -> DBResponse<Vec<T>>
    where T: BrussType, Q: DBQuery
{
    match db
        .database(CONFIGS.db.get_db())
        .collection::<T>(T::DB_NAME)
        .find(query.to_doc(), None)
        .await
    {
        Ok(found) => found.try_collect::<Vec<T>>().await.into(),
        Err(e) => DBResponse::DBError(e.to_string())
    }
}



