#![doc = include_str!("../README.md")]

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use authority::Authority;
use base64::{engine::general_purpose, Engine};
use error::Error;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod authority;
pub mod error;

const CLIENT_ID: &str = "client_id";
const SCOPES: &str = "scope";
const GRANT_TYPE: &str = "grant_type";
const CLIENT_CREDENTIALS_GRANT: &str = "client_credentials";
const CLIENT_SECRET: &str = "client_secret";
const ASSERTION: &str = "client_assertion";
const ASSERTION_TYPE: &str = "client_assertion_type";
const CLIENT_ASSERTION_GRANT_TYPE: &str = "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";

pub enum ClientCredential {
    ClientSecret(String),
    Certificate(Certificate),
}

impl ClientCredential {
    /// Create a new client credential from a client secret.
    ///
    /// See: [1-Call-MsGraph-WithSecret](https://github.com/Azure-Samples/ms-identity-python-daemon/blob/master/1-Call-MsGraph-WithSecret/README.md)
    pub fn from_secret(secret: String) -> Self {
        ClientCredential::ClientSecret(secret)
    }

    /// Create a new client credential from a certificate.
    ///
    /// See: [2-Call-MsGraph-WithCertificate](https://github.com/Azure-Samples/ms-identity-python-daemon/blob/master/2-Call-MsGraph-WithCertificate/README.md)
    pub fn from_certificate(private_key: Vec<u8>, thumbprint: String) -> Self {
        ClientCredential::Certificate(Certificate {
            private_key,
            thumbprint,
        })
    }
}

pub struct Certificate {
    private_key: Vec<u8>,
    thumbprint: String,
}

pub struct ConfidentialClient {
    client_id: String,
    authority: Authority,
    credential: ClientCredential,
}

impl ConfidentialClient {
    pub async fn new(
        client_id: &str,
        authority: &str,
        credential: ClientCredential,
    ) -> Result<ConfidentialClient, Error> {
        let authority = Authority::new(authority).await?;

        Ok(ConfidentialClient {
            client_id: client_id.to_string(),
            authority,
            credential,
        })
    }

    pub async fn acquire_token_silent(&mut self, scopes: &[&str]) -> Result<TokenResponse, Error> {
        let mut params = HashMap::new();

        let assertion;
        match &self.credential {
            ClientCredential::ClientSecret(client_secret) => {
                params.insert(CLIENT_SECRET, client_secret.as_str());
            }
            ClientCredential::Certificate(certificate) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let audience = &self.authority.token_endpoint;
                let issuer = &self.client_id;
                let uuid = Uuid::new_v4().to_string();
                let claims = AssertionClaims {
                    aud: audience,
                    sub: issuer,
                    iss: issuer,
                    exp: 600 + now,
                    iat: now,
                    jti: uuid.as_str(),
                };

                let encoding_key = EncodingKey::from_rsa_pem(&certificate.private_key)?;
                let mut header = Header::new(Algorithm::RS256);

                let sha1_thumbprint =
                    general_purpose::STANDARD.encode(hex::decode(&certificate.thumbprint)?);
                header.x5t = Some(sha1_thumbprint);

                assertion = encode(&header, &claims, &encoding_key)?;

                params.insert(ASSERTION, &assertion);
                params.insert(ASSERTION_TYPE, CLIENT_ASSERTION_GRANT_TYPE);
            }
        }

        params.insert(CLIENT_ID, &self.client_id);
        let scope = scopes.join(" ");
        params.insert(SCOPES, &scope);
        params.insert(GRANT_TYPE, CLIENT_CREDENTIALS_GRANT);

        let response = reqwest::Client::new()
            .post(&self.authority.token_endpoint)
            .form(&params)
            .send()
            .await?;

        let response: TokenResponse = response.json().await?;

        Ok(response)
    }
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub expires_in: Option<u64>,
    pub ext_expires_in: Option<u64>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,

    // Error
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub error_codes: Option<Vec<usize>>,
    pub timestamp: Option<String>,
    pub trace_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct AssertionClaims<'a> {
    aud: &'a str,
    sub: &'a str,
    iss: &'a str,
    jti: &'a str,
    iat: u64,
    exp: u64,
}
