use reqwest::Client;
use serde::Deserialize;

use crate::error::Error;

const TENANT_DISCOVERY_ENDPOINT: &str = "/v2.0/.well-known/openid-configuration";

pub(crate) struct Authority {
    pub(crate) token_endpoint: String,
}

impl Authority {
    pub(crate) async fn new(authority_url: &str, client: &Client) -> Result<Self, Error> {
        let response = client
            .get(&format!("{}{}", authority_url, TENANT_DISCOVERY_ENDPOINT))
            .send()
            .await?;
        let response: TenantDiscoveryResponse = response.json().await?;

        Ok(Authority {
            token_endpoint: response.token_endpoint,
        })
    }
}

#[derive(Deserialize)]
struct TenantDiscoveryResponse {
    token_endpoint: String,
}
