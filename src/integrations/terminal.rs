use async_trait::async_trait;

use crate::{
    integrations::Notifier,
    models::{ChannelId, Listing, Subscription},
    parsers::ScrapeMetadata,
};

/// A notifier that prints the listing to the terminal. Intended for testing, but it's
/// fully functional.
pub struct TerminalAdapter;
impl TerminalAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Notifier for TerminalAdapter {
    async fn notify_new_listing(
        &self,
        subscription: &Subscription,
        metadata: &ScrapeMetadata,
        listing: &Listing,
        channel_id: ChannelId,
    ) -> Result<(), String> {
        println!("New listing: {} {}", listing.title, listing.url);
        // println!("{listing:?}");
        // Can't really fail
        Ok(())
    }

    async fn send_coconut(&self, channel_id: ChannelId) -> Result<(), String> {
        Ok(())
    }
}
