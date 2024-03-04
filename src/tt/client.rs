use super::{route::TTRoute, TTArea, TTError, TTResult, ToBruss};
use reqwest::{Client, Request, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize};

pub struct TTClient {
    base_url: String,
    secret: String,
    client: Client
}

impl TTClient {
    pub fn new(base_url: String, secret: String) -> Self {
        Self { base_url, secret, client: Client::new() }
    }

    fn auth_req(&self, url: &str) -> RequestBuilder {
        self.client
            .get(format!("{}{}", self.base_url, url))
            .header("authorization", format!("Basic {}", self.secret))
    }

    async fn get_data<T>(&self, r: RequestBuilder) -> TTResult<Vec<T>> where T: ToBruss + DeserializeOwned {
        r
        // Ok(r
            .send()
            .await?
            .json::<Vec<T>>()
            .await
            // .unwrap())
            .map_err(TTError::from)
    }

    pub async fn get_areas(&self) -> TTResult<Vec<TTArea>> {
        self.get_data(self.auth_req("areas")).await
    }

    pub async fn get_routes(&self) -> TTResult<Vec<TTRoute>> {
        self.get_data(self.auth_req("routes")).await
    }
}

