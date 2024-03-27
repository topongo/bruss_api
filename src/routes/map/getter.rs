use futures::TryStreamExt;
use rocket::serde::json::Json;
use rocket::http::Status;
use mongodb::bson::Document;
use rocket_db_pools::Connection;
use bruss_config::CONFIGS;
use bruss_data::BrussType;
use crate::db::BrussData;

#[derive(Responder)]
pub enum GetResponse<T> {
    #[response(status = 200)]
    Ok { inner: (Status, Json<T>) },
    #[response(status = 500, content_type = "json")]
    SerializationError(String),
    #[response(status = 500)]
    DBError(String),
}

impl<T> From<Result<T, mongodb::error::Error>> for GetResponse<T> {
    fn from(value: Result<T, mongodb::error::Error>) -> Self {
        match value {
            Ok(v) => GetResponse::Ok {
                inner: (Status::Ok, Json(v)),
            },
            Err(e) => {
                error!("Internal Server Error: {:?}", e);
                GetResponse::DBError("error fetching data".to_owned())
            },
        }
    }
} 

pub trait GetterQuery {
    fn to_doc(self) -> Document;
}

pub async fn get<Q, T>(db: Connection<BrussData>, query: Q) -> GetResponse<Vec<T>>
    where T: BrussType, Q: GetterQuery
{
    match db
        .database(CONFIGS.db.get_db())
        .collection::<T>(T::DB_NAME)
        .find(query.to_doc(), None)
        .await
    {
        Ok(found) => found.try_collect::<Vec<T>>().await.into(),
        Err(e) => GetResponse::DBError(e.to_string())
    }
}



