use msal_rs::{ClientCredential, ConfidentialClient};

#[tokio::main]
async fn main() {
    // See <https://github.com/Azure-Samples/ms-identity-python-daemon> to get started.

    // From secret:
    let client_credential = ClientCredential::from_secret("secret".to_string());

    // From certificate:
    // let client_credential = ClientCredential::from_certificate(
    //     include_bytes!("path/server.pem").to_vec(),
    //     "thumbprint".to_string(),
    // );

    let authority = "https://login.microsoftonline.com/:tenant_id";
    let client_id = "uuid";
    let scopes = vec!["https://graph.microsoft.com/.default"];

    let mut app = ConfidentialClient::new(client_id, authority, client_credential)
        .await
        .unwrap();

    let resp = app.acquire_token_silent(&scopes).await.unwrap();
    let token = resp.access_token.unwrap();
    println!("Token: {}", token);

    let expires_in = resp.expires_in.unwrap();
    println!("Expires in: {}", expires_in);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();

    let resp = client
        .get("https://graph.microsoft.com/v1.0/users")
        .send()
        .await
        .unwrap();
    println!("{:?}", resp.json::<serde_json::Value>().await.unwrap());
}
