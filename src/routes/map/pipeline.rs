use bruss_config::CONFIGS;
use mongodb::bson::{doc, Document};

use super::query::DBQuery;

#[derive(Debug)]
pub struct Pipeline {
    find: Document,
    limit: i64,
    skip: i64,
    pre_sort: Document,
    sort: Document,
}

impl Pipeline {
    pub fn new(find: Document) -> Self {
        Self {
            find,
            skip: 0,
            limit: CONFIGS.api.default_limit,
            pre_sort: doc!{},
            sort: doc!{"_id": 1},
        }
    }

    pub fn build(self) -> BuiltPipeline {
        BuiltPipeline::from(self)
    }

    pub fn limit(mut self, limit: Option<u32>) -> Self {
        if let Some(limit) = limit {
            if !(0..=100).contains(&limit) {
                self.limit = CONFIGS.api.default_limit;
            } else {
                self.limit = limit as i64;
            }
        }
        self
    }

    pub fn skip(mut self, skip: Option<u32>) -> Self {
        if let Some(skip) = skip {
            self.skip = skip as i64;
        }
        self
    }

    pub fn pre_sort(mut self, pre_sort: Document) -> Self {
        self.pre_sort = pre_sort;
        self
    }

    pub fn sort(mut self, sort: Document) -> Self {
        self.sort = sort;
        self
    }

    pub fn query(self) -> Document {
        self.find
    }
}

impl<Q: DBQuery> From<Q> for Pipeline {
    fn from(query: Q) -> Self {
        Self::new(query.to_doc())
    }
}

impl From<Document> for Pipeline {
    fn from(doc: Document) -> Self {
        Self::new(doc)
    }
}

pub(crate) struct BuiltPipeline {
    pub count: Vec<Document>,
    pub fetch: Vec<Document>,
}

impl From<Pipeline> for BuiltPipeline {
    fn from(value: Pipeline) -> Self {
        let mut fetch = vec![];
        let mut count = vec![];
        if !value.find.is_empty() {
            fetch.push(doc!{"$match": value.find.clone()});
            count.push(doc!{"$match": value.find});
        }
        if !value.pre_sort.is_empty() {
            fetch.push(value.pre_sort);
        }
        if !value.sort.is_empty() {
            fetch.push(doc!{"$sort": value.sort});
        }
        fetch.push(doc!{"$skip": value.skip});
        fetch.push(doc!{"$limit": value.limit});
        count.push(doc!{"$count": "count"});
        info!("  Generated pipeline: {:?}", fetch);
        BuiltPipeline {
            count,
            fetch
        }
    }
}

