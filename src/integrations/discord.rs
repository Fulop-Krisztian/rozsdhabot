use crate::{
    integrations::Notifier,
    models::{ChannelId, Listing, Subscription},
    parsers::ScrapeMetadata,
};

pub struct DiscordAdapter {
    token: String,
}

impl DiscordAdapter {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
}

use async_trait::async_trait;
#[async_trait]
impl Notifier for DiscordAdapter {
    async fn notify_new_listing(
        &self,
        subscription: &Subscription,
        metadata: &ScrapeMetadata,
        listing: &Listing,
        channel_id: ChannelId,
    ) -> Result<(), String> {
        // match channel_id {
        //     ChannelId::Discord {
        //         guild_id,
        //         channel_id,
        //     } => {
        //         todo!()
        //     }
        //     _ => Err("Invalid channel ID: expected Discord channel.".to_string()),
        // }

        todo!()
    }
    async fn send_coconut(&self, channel_id: ChannelId) -> Result<(), String> {
        todo!()
    }
}
