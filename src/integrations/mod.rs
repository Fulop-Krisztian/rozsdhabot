mod discord;
mod message_handler;
mod telegram;
mod terminal;
use std::sync::Arc;

use crate::{
    AppCtx,
    models::{ChannelId, Listing, Subscription},
    parsers::ScrapeMetadata,
};

pub use self::{
    discord::DiscordController, discord::DiscordNotifier, telegram::TelegramIntegration,
    terminal::TerminalIntegration,
};

/// A struct that contains all implemented notifiers.
///
/// Disabled adapters are represented by `None`.
///
/// It's very highly recommended to pass an adapter that is the type that is indicated by the name, but this is not enforced.
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
/// A notifier is responsible for sending messages to a channel.
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

/// A controller is responsible for receiving messages from a channel and mutating the application
/// state accordingly.
#[async_trait]
pub trait Controller: Send {
    async fn start(self: Box<Self>, app_context: AppCtx) -> ();
}

// ==== Recieving messages ====
