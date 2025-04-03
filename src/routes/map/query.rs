use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::Document, Collection};
use rocket_db_pools::Connection;
use bruss_config::CONFIGS;
use bruss_data::{BrussType, Schedule};
use serde::{de::DeserializeOwned, Deserialize};
use tokio::time::Instant;
use crate::db::BrussData;
use mongodb::error::Error as MongoError;
use super::{pipeline::{BuiltPipeline, Pipeline}, trip::{TripAtStop, TripInRoute}};

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

/// Interface for a database interface that can be used to obtain a specific collection of data
/// implementing the BrussType trait.
pub trait Collectable {
    // fn get_coll<T: BrussType>(&self) -> Collection<T>;
    fn get_coll_raw<T: BrussType, O>(&self) -> Collection<O>;
}

impl Collectable for DBInterface {
    // fn get_coll<T: BrussType>(&self) -> Collection<T> {
    //     T::get_coll(&self.0.database(CONFIGS.db.get_db()))
    // }
    fn get_coll_raw<T: BrussType, O>(&self) -> Collection<O> {
        self.0.database(CONFIGS.db.get_db()).collection::<O>(T::TYPE.collection())
    }
}

/// Trait for querying the database, using a type `T` for data output and a type `X` for the input
/// collection, mainly used for cross-collection queries.
///
/// For example: using the collection `UserPermissions` to get a list of `User` objects.
///
/// This can and must be also reflective: implementation of `Queryable<T, T>` for a type will
/// grant it the ability to query the collection of type `T` and getting results of type `T`.
pub trait Queryable<T, X>: Collectable
where
    T: DeserializeOwned + Sync + Unpin + Send,
    X: BrussType + Sync + Unpin + Send,
{
    async fn query(&self, pipeline: impl Into<BuiltPipeline>) -> Result<QueryResult<T>, MongoError> {
        let pipeline: BuiltPipeline = pipeline.into();
        let start = Instant::now();
        let count = match self.get_coll_raw::<X, Vec<i64>>()
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
        let elapsed = start.elapsed();
        if elapsed.as_secs_f32() > 0.5 {
            log::warn!("count stage took {:?}", start.elapsed());
        }
        let start = Instant::now();
        let r = match self.get_coll_raw::<X, T>()
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
        };
        let elapsed = start.elapsed();
        if elapsed.as_secs_f32() > 0.5 {
            log::warn!("fetch stage took {:?}", start.elapsed());
        }
        r
    }

    async fn query_single(&self, pipeline: impl Into<BuiltPipeline>) -> Result<Option<T>, MongoError> {
        self.get_coll_raw::<X, T>()
            .find_one(pipeline.into().query(), None)
            .await
    }

    #[allow(dead_code)]
    async fn query_db<Q: DBQuery>(&self, query: Q) -> Result<QueryResult<T>, MongoError> {
        Self::query(self, Pipeline::from(query)).await
    }
}

/// Trait for querying the database, in the specific case in which the collection type and the
/// output type are the same.
/// It's defined as `trait UniformQueryable<T>: Queryable<T, T>` for allowing this.
///
/// Automatically implemented for any type `T` implementing the `BrussType` trait (and also
/// `Sync`, `Unpin` and `Send` for async purposes).
///
/// This trait shadows the `query` and `query_single` methods of the `CrossQueryable` trait, so
/// that an object that implements `Queryable<T>` can be used directly instead of using a trait
/// object.
pub trait UniformQueryable<T>: Queryable<T, T> where T: BrussType + Sync + Unpin + Send {
    async fn query(&self, pipeline: impl Into<BuiltPipeline>) -> Result<QueryResult<T>, MongoError> {
        Queryable::query(self, pipeline).await
    }

    async fn query_single(&self, pipeline: impl Into<BuiltPipeline>) -> Result<Option<T>, MongoError> {
        Queryable::query_single(self, pipeline).await
    }
}

/// Reflective implementation of the `Queryable` trait for any type `T` implementing BrussType.
impl<T: BrussType + Sync + Unpin + Send> Queryable<T, T> for DBInterface {}
impl<T: BrussType + Sync + Unpin + Send> UniformQueryable<T> for DBInterface {}

/// Implementation of the `CrossQueryable` trait for `DBInterface` in types `Schedule` with return
/// type `TripInRoute` and `TripAtStop`.
impl Queryable<TripInRoute, Schedule> for DBInterface {}
impl Queryable<TripAtStop, Schedule> for DBInterface {}

#[derive(Deserialize)]
struct CountResult {
    count: i64,
}

#[derive(Debug)]
pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub total: usize,
}

