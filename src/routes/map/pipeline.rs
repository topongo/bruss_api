use bruss_config::CONFIGS;
use mongodb::bson::{doc, Document};

use super::query::DBQuery;

#[derive(Debug)]
pub struct Pipeline {
    find: Document,
    limit: i64,
    pre_sort: Document,
    sort: Document,
}

impl Pipeline {
    pub fn new(find: Document) -> Self {
        Self {
            find,
            limit: CONFIGS.api.default_limit,
            pre_sort: doc!{},
            sort: doc!{"_id": 1},
        }
    }

    pub fn build(self) -> Vec<Document> {
        let mut pipeline = vec![];
        if !self.find.is_empty() {
            pipeline.push(doc!{"$match": self.find});
        }
        if !self.pre_sort.is_empty() {
            pipeline.push(self.pre_sort);
        }
        if !self.sort.is_empty() {
            pipeline.push(doc!{"$sort": self.sort});
        }
        pipeline.push(doc!{"$limit": self.limit});
        info!("  Generated pipeline: {:?}", pipeline);
        pipeline
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
