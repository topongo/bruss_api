use futures::TryStreamExt;
// use rocket::serde::json::Json;
use mongodb::bson::Document;
use rocket_db_pools::Connection;
use bruss_config::CONFIGS;
use bruss_data::BrussType;
use crate::db::BrussData;
use mongodb::error::Error as MongoError;

pub trait DBQuery {
    fn to_doc(self) -> Document;
}

pub struct RawDBQuery(pub Document);

impl DBQuery for RawDBQuery {
    fn to_doc(self) -> Document {
        self.0
    }
}

pub struct DBInterface(pub Connection<BrussData>);

// #[allow(dead_code)]
pub trait Queriable<T> {
    async fn query(&self, query: Document) -> Result<T, MongoError>;

    async fn query_db<Q: DBQuery>(&self, query: Q) -> Result<T, MongoError> {
        Self::query(&self, query.to_doc()).await
    }

    // async fn query_json<Q: DBQuery>(&self, query: Json<Q>) -> Result<T, MongoError> {
    //     Self::query(&self, query.into_inner().to_doc()).await
    // }
}

impl<T: BrussType> Queriable<Vec<T>> for DBInterface {
    async fn query(&self, query: Document) -> Result<Vec<T>, MongoError> {
        match T::get_coll(&self.0.database(CONFIGS.db.get_db()))
            .find(query, None)
            .await 
        {
            Ok(found) => found.try_collect::<Vec<T>>().await,
            Err(e) => Err(e)
        }
    }
}

impl<T: BrussType + Sync + Unpin + Send> Queriable<Option<T>> for DBInterface {
    async fn query(&self, query: Document) -> Result<Option<T>, MongoError> {
        T::get_coll(&self.0.database(CONFIGS.db.get_db()))
            .find_one(query, None)
            .await
    }
}

