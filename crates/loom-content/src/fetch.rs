//! HTTP client with TLS

use reqwest::Client;

/// HTTP client
#[derive(Debug)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .use_rustls_tls()
            .build()?;
        
        Ok(Self { client })
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<String> {
        let response = self.client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }
}
