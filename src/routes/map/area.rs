use std::fmt::Debug;

use lazy_static::lazy_static;
use rocket::form::Strict;
use mongodb::bson::{doc, Document};
use bruss_data::Area;
use super::{AreaTypeWrapper,query::DBQuery,gen_generic_getters};
use super::pipeline::Pipeline;


#[derive(FromForm,Debug)]
pub struct AreaQuery {
    // id: Strict<Option<u16>>,
    #[field(name = "type")]
    ty: Strict<Option<AreaTypeWrapper>>,
}

impl DBQuery for AreaQuery {
    fn to_doc(self) -> Document {
        let mut d = Document::new();
        if let Some(ty) = self.ty.into_inner() { d.insert::<_, &'static str>("type", ty.into()); }
        error!("{d:?}");
        d
    }
}

gen_generic_getters!(Area, AreaQuery, u16);

lazy_static!{
    pub static ref ROUTES: Vec<rocket::Route> = routes![get, get_opts];
}

