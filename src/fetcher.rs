/// This file contains the fetcher for the bot.
/// The user agent is randomized to work around rate limiting.
// hardverapro.hu robots.txt reads as follows:
// User-agent: *
//
// crawl-delay: 1
use reqwest::Client;

#[derive(Debug)]
pub struct Fetcher {
    client: reqwest::Client,
}

impl Fetcher {
    /// Instead of throwing out new requests left and right.
    pub fn new() -> Self {
        const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

        let client = Client::builder()
            .timeout(TIMEOUT)
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
