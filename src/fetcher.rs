use reqwest::Client;
/**
* @file fetcher.rs
* @brief Fetches listings from Hardverapro. Uses a random user agent for requests
* @author Fülöp Krisztián Szilárd
* @date 2025-12-02
*/
use std::time::Duration;

pub struct Fetcher {
    client: reqwest::Client,
}

impl Fetcher {
    /// Instead of throwing out new requests left and right.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            // Fake use agent to stay under the rader.
            // The agent differs beteween rozsdhabot restarts.
            .user_agent(fake_user_agent::get_rua())
            .build()
            // unsafe for now
            .unwrap();
        Self { client }
    }

    pub async fn fetch(&self, url: &str) -> Result<String, reqwest::Error> {
        let response = self.client.get(url).send().await?;
        match response.error_for_status_ref() {
            Ok(_) => {
                let body = response.text().await?;
                Ok(body)
            }
            Err(e) => Err(e),
        }
    }
}
