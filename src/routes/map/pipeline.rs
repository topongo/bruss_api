use std::fmt::Display;

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

#[allow(dead_code)]
impl Pipeline {
    pub fn new(find: Document) -> Self {
        Self {
            find,
            skip: 0,
            limit: Self::default_limit(),
            pre_sort: doc!{},
            sort: doc!{"_id": 1},
        }
    }

    pub fn default_limit() -> i64 {
        CONFIGS.api.default_limit
    }

    pub fn build(self) -> BuiltPipeline {
        BuiltPipeline::from(self)
    }

    pub fn limit(mut self, limit: Option<u32>) -> Self {
        if let Some(limit) = limit {
            if !(0..=100).contains(&limit) {
                self.limit = Self::default_limit();
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

    pub fn custom(fetch: Vec<Document>, count: Vec<Document>) -> CustomPipeline {
        CustomPipeline { fetch, count }
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

#[derive(Debug)]
pub(crate) struct CustomPipeline {
    fetch: Vec<Document>,
    count: Vec<Document>,
}

impl Display for CustomPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomPipeline {{\n\tfetch: [\n{}]\n\tcount: [\n{}]\n}}",
            self.fetch.iter().map(|d| d.to_string()).collect::<Vec<String>>().join(",\n"),
            self.count.iter().map(|d| d.to_string()).collect::<Vec<String>>().join(",\n"),
        )
    }
}

pub(crate) struct BuiltPipeline {
    pub count: Vec<Document>,
    pub fetch: Vec<Document>,
    pub query: Option<Document>,
}

impl BuiltPipeline {
    pub fn query(self) -> Document {
        if self.query.is_none() {
            panic!("custom pipelines cannot be used to query a single document");
        }
        self.query.unwrap()
    }
}

impl From<Pipeline> for BuiltPipeline {
    fn from(value: Pipeline) -> Self {
        let mut fetch = vec![];
        let mut count = vec![];
        if !value.find.is_empty() {
            fetch.push(doc!{"$match": value.find.clone()});
            count.push(doc!{"$match": value.find.clone()});
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
            fetch,
            query: Some(value.find),
        }
    }
}

impl From<CustomPipeline> for BuiltPipeline {
    fn from(value: CustomPipeline) -> Self {
        BuiltPipeline {
            count: value.count,
            fetch: value.fetch,
            query: None,
        }
    }
}
