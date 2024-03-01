use super::{TTArea,TTError};
use reqwest::{Client, RequestBuilder};

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

    pub async fn get_areas(&self) -> Result<Vec<TTArea>, TTError> {
        self.auth_req("areas")
            .send()
            .await?
            .json::<Vec<TTArea>>()
            .await
            .map_err(TTError::from)
    }        
}

