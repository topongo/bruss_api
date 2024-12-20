pub mod query;
pub mod params;
pub mod area;
pub mod route;
pub mod stop;
pub mod trip;
pub mod path;
pub mod segment;
pub mod pipeline;

// pub use route::{get_route,get_route_opt};
// pub use stop::{get_stop,get_stop_opt};
// pub use segment::get_segments;
// pub use trip::{get_trips_route,get_trips_stop};
// pub use path::get_path;

use mongodb::bson::Bson;
use tt::AreaType;
use rocket::form::FromFormField;
use rocket::form::Error;
use serde::{Deserialize,Serialize};

#[derive(Deserialize,Serialize,Clone,Debug)]
#[serde(transparent)]
pub struct AreaTypeWrapper {
    inner: AreaType
}

impl FromFormField<'_> for AreaTypeWrapper {
    fn from_value(field: rocket::form::ValueField<'_>) -> rocket::form::Result<'_,Self> {
        field.value.parse()
            .map_err(|_| Error::validation("could be either \"u\" or \"e\"").into())
            .map(|v| AreaTypeWrapper { inner: v }) 
    }
}

impl Into<&'static str> for AreaTypeWrapper {
    fn into(self) -> &'static str {
        self.inner.into()
    }
}

impl Into<Bson> for AreaTypeWrapper {
    fn into(self) -> Bson {
        <AreaType as Into<&str>>::into(self.inner).into()
    }
}

macro_rules! gen_generic_getters {
    ($type:ident, $query:ty, $id_type:ident) => {
        use crate::routes::map::query::QueryResult;

        #[get("/<id>")]
        pub async fn get(
            db: rocket_db_pools::Connection<crate::BrussData>,
            id: Result<super::params::Id<$id_type>, <super::params::Id<$id_type> as rocket::request::FromParam<'_>>::Error>,
        ) -> crate::response::ApiResponse<$type> {
            super::query::Queriable::<Option<$type>>::query(&super::query::DBInterface(db), id?.to_doc().into()).await.into()
        }

        #[get("/?<limit>&<skip>&<query..>")]
        pub async fn get_opts(
            db: rocket_db_pools::Connection<crate::BrussData>, 
            query: rocket::form::Result<'_, rocket::form::Strict<$query>>,
            limit: Option<u32>,
            skip: Option<u32>,
        ) -> crate::response::ApiResponse<Vec<$type>> {
            println!("id: {:?}", query);
            super::query::Queriable::<QueryResult<$type>>::query(
                &super::query::DBInterface(db), 
                Pipeline::from(query?.into_inner())
                    .limit(limit)
                    .skip(skip)
            ).await.into()
        }
    };
}

macro_rules! gen_area_getters {
    ($type:ident, $query:ty, $id_type:ident) => {
        use crate::routes::map::query::QueryResult;

        #[get("/<area_type>/<id>?<limit>")]
        pub async fn get(
            db: rocket_db_pools::Connection<crate::BrussData>, 
            area_type: Result<super::params::Id<super::AreaTypeWrapper>, <super::params::Id<super::AreaTypeWrapper> as rocket::request::FromParam<'_>>::Error>,
            id: Result<super::params::Id<$id_type>, <super::params::Id<$id_type> as rocket::request::FromParam<'_>>::Error>,
            limit: Option<u32>
        ) -> crate::response::ApiResponse<$type> {
            let mut d = id?.to_doc();
            d.insert("type", area_type?.value());
            Queriable::<Option<$type>>::query(&DBInterface(db), Pipeline::from(d).limit(limit)).await.into()
        }

        #[get("/?<skip>&<limit>&<query..>")]
        pub async fn get_opts(
            db: Connection<BrussData>, 
            query: rocket::form::Result<'_, Strict<$query>>,
            skip: Option<u32>,
            limit: Option<u32>,
        ) -> ApiResponse<Vec<$type>> {
            Queriable::<QueryResult<$type>>::query(&DBInterface(db), Pipeline::from(query?.into_inner()).skip(skip).limit(limit)).await.into()
        }
    };
}

pub(crate) use {gen_generic_getters,gen_area_getters};

