use crate::configs::CONFIGS;
use crate::data::ToBruss;
use super::{route::TTRoute, TTArea, TTError, TTResult};
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

#[tokio::test]
async fn auth_test() {
    use crate::configs::CONFIGS;

    let tt = CONFIGS.tt.client();
    let response = tt.auth_req("areas")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn areas_parse_test() {
    use crate::configs::CONFIGS;

    let tt = CONFIGS.tt.client();
    let areas: Vec<TTArea> = tt
        .auth_req("areas")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(areas.len() > 0);
}

#[tokio::test]
async fn routes_parse_test() {
    use crate::configs::CONFIGS;
   
    let tt = CONFIGS.tt.client();
    let routes: Vec<TTRoute> = tt
        .auth_req("routes")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(routes.len() > 0);
}

#[test]
fn auth_req_test() {
}

