pub mod query;
pub mod params;
pub mod area;
pub mod route;
pub mod stop;
mod trip;
pub mod path;
pub mod segment;

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
        #[get("/<id>")]
        pub async fn get(
            db: rocket_db_pools::Connection<crate::BrussData>,
            id: Result<super::params::Id<$id_type>, <super::params::Id<$id_type> as rocket::request::FromParam<'_>>::Error>
        ) -> crate::response::ApiResponse<$type> {
            super::query::Queriable::<Option<$type>>::query(&super::query::DBInterface(db), id?.to_doc()).await.into()
        }

        #[get("/?<query..>")]
        pub async fn get_opts(
            db: rocket_db_pools::Connection<crate::BrussData>, 
            query: rocket::form::Result<'_, rocket::form::Strict<$query>>
        ) -> crate::response::ApiResponse<Vec<$type>> {
            super::query::Queriable::<Vec<$type>>::query_db(&super::query::DBInterface(db), query?.into_inner()).await.into()
        }
    };
}

macro_rules! gen_area_getters {
    ($type:ident, $query:ty, $id_type:ident) => {
        #[get("/<area_type>/<id>")]
        pub async fn get(
            db: rocket_db_pools::Connection<crate::BrussData>, 
            area_type: Result<super::params::Id<super::AreaTypeWrapper>, <super::params::Id<super::AreaTypeWrapper> as rocket::request::FromParam<'_>>::Error>,
            id: Result<super::params::Id<$id_type>, <super::params::Id<$id_type> as rocket::request::FromParam<'_>>::Error>
        ) -> crate::response::ApiResponse<$type> {
            let mut d = id?.to_doc();
            d.insert("type", area_type?.value());
            Queriable::<Option<$type>>::query(&DBInterface(db), d).await.into()
        }

        #[get("/?<query..>")]
        pub async fn get_opts(db: Connection<BrussData>, query: rocket::form::Result<'_, Strict<$query>>) -> ApiResponse<Vec<$type>> {
            Queriable::<Vec<$type>>::query_db(&DBInterface(db), query?.into_inner()).await.into()
        }
    };
}

pub(crate) use {gen_generic_getters,gen_area_getters};

