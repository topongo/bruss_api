use futures::{StreamExt, TryStreamExt};
use mongodb::bson::Document;
use rocket_db_pools::Connection;
use bruss_config::CONFIGS;
use bruss_data::BrussType;
use serde::Deserialize;
use crate::db::BrussData;
use mongodb::error::Error as MongoError;

use super::pipeline::Pipeline;

/// Allow struct to be converted to a mongodb query.
pub trait DBQuery {
    fn to_doc(self) -> Document;
}

/// Generic struct, wrapper for a mongodb Document to implement DBQuery.
pub struct RawDBQuery(pub Document);

impl DBQuery for RawDBQuery {
    fn to_doc(self) -> Document {
        self.0
    }
}

/// Interface for routes, to query the database.
pub struct DBInterface(pub Connection<BrussData>);

// #[allow(dead_code)]
pub trait Queriable<T> {
    async fn query(&self, pipeline: Pipeline) -> Result<T, MongoError>;

    #[allow(dead_code)]
    async fn query_db<Q: DBQuery>(&self, query: Q) -> Result<T, MongoError> {
        Self::query(self, Pipeline::from(query)).await
    }
    // async fn query_json<Q: DBQuery>(&self, query: Json<Q>) -> Result<T, MongoError> {
    //     Self::query(&self, query.into_inner().to_doc()).await
    // }
}

#[derive(Deserialize)]
struct CountResult {
    count: i64,
}

impl<T: BrussType> Queriable<QueryResult<T>> for DBInterface {
    async fn query(&self, pipeline: Pipeline) -> Result<QueryResult<T>, MongoError> {
        let pipeline = pipeline.build();
        let count = match self.0.database(CONFIGS.db.get_db()).collection::<Vec<i64>>(T::TYPE.collection())
            .aggregate(pipeline.count, None)
            .await
        {
            Ok(c) => {
                let res: Vec<CountResult> = c.map(|i| mongodb::bson::from_document::<CountResult>(i.unwrap())).try_collect().await.unwrap();
                if res.is_empty() {
                    0
                } else {
                    res[0].count
                }
            },
            Err(e) => return Err(e)
        } as usize;
        match T::get_coll(&self.0.database(CONFIGS.db.get_db()))
            .aggregate(pipeline.fetch, None)
            .await
        {
            Ok(found) => {
                found.map(|i| mongodb::bson::from_document(i.unwrap()))
                    .try_collect()
                    .await
                    .map(|result| QueryResult {
                        data: result,
                        total: count,
                    })
                    .map_err(MongoError::from)
            }
            Err(e) => Err(e)
        }
    }
}

impl<T: BrussType + Sync + Unpin + Send> Queriable<Option<T>> for DBInterface {
    async fn query(&self, pipeline: Pipeline) -> Result<Option<T>, MongoError> {
        T::get_coll(&self.0.database(CONFIGS.db.get_db()))
            .find_one(pipeline.query(), None)
            .await
    }
}

pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub total: usize,
}


