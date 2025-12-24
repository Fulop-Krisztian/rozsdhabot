mod discord;
mod message_handler;
mod telegram;
mod terminal;
use std::sync::Arc;

use crate::{
    models::{ChannelId, Listing, Subscription},
    parsers::ScrapeMetadata,
};

pub use self::{
    discord::DiscordAdapter, telegram::TelegramAdapter, telegram::telegram_handler,
    terminal::TerminalAdapter,
};

/// A struct that contains all implemented notifiers.
/// It's very highly recommended to pass an adapter that is the type of the name, but this is not
/// enforced.
///
/// Anything can be passed to any notifier
#[derive(Default, Clone)]
pub struct NotifierRegistry {
    pub telegram: Option<Arc<dyn Notifier>>,
    pub discord: Option<Arc<dyn Notifier>>,
    pub terminal: Option<Arc<dyn Notifier>>,
}

impl NotifierRegistry {
    pub fn notifier_for(&self, channel: &ChannelId) -> Option<Arc<dyn Notifier>> {
        match channel {
            ChannelId::Telegram { .. } => self.telegram.clone(),
            ChannelId::Discord { .. } => self.discord.clone(),
            ChannelId::Terminal => self.terminal.clone(),
        }
    }
}

// ==== Sending messages ====
use async_trait::async_trait;
#[async_trait]
pub trait Notifier: Send + Sync {
    async fn notify_new_listing(
        &self,
        subscription: &Subscription,
        metadata: &ScrapeMetadata,
        listing: &Listing,
        channel_id: ChannelId,
    ) -> Result<(), String>;
    async fn send_coconut(&self, channel_id: ChannelId) -> Result<(), String>;
}

// ==== Recieving messages ====
