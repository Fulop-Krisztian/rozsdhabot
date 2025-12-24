use std::sync::{Arc, Mutex};

use crate::{
    integrations::{
        Notifier, NotifierRegistry,
        message_handler::{IncomingMessage, IncomingMessageHandler},
    },
    models::{ChannelId, Listing, Subscription},
    monitor::MonitorManager,
    parsers::ScrapeMetadata,
    storage::{RuntimeStateStore, SubscriptionStore},
};

use async_trait::async_trait;
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    prelude::{Requester, ResponseResult},
    types::{InputFile, Message},
};

pub struct TelegramAdapter {
    pub bot: teloxide::Bot,
}

impl TelegramAdapter {
    pub fn new(bot: teloxide::Bot) -> Self {
        Self { bot }
    }

    fn format_notification(
        &self,
        sub: &Subscription,
        metadata: &ScrapeMetadata,
        listing: &Listing,
    ) -> String {
        use teloxide::utils::markdown;

        let link = markdown::link(&listing.url, markdown::escape(&listing.title).as_str());

        let sub_title = markdown::link(
            &sub.url,
            markdown::escape(sub.name.clone().unwrap_or("(unnamed)".to_string()).as_str()).as_str(),
        );

        let id = sub.id.to_string();

        let price = markdown::bold(markdown::escape(listing.price.to_string().as_str()).as_str());
        let seller_name = markdown::escape(listing.seller_name.as_str());
        let seller_ratings = markdown::escape(listing.seller_ratings.to_string().as_str());
        let cities = markdown::escape(listing.cities.join(", ").as_str());

        let pricerange = markdown::escape(
            match (metadata.min_price, metadata.max_price) {
                (Some(min), Some(max)) => format!("between {min:.0} - {max:.0} Ft"),
                (Some(min), None) => format!("above {min:.0} Ft"),
                (None, Some(max)) => format!("under {max:.0} Ft"),
                (None, None) => "".to_string(),
            }
            .as_str(),
        );

        format!(
            "
{price} Ft
{link}
{cities}
\\- {seller_name} \\(\\+{seller_ratings}\\)

From subscription:
{sub_title} \\({id}\\):
{pricerange}
",
        )
    }
}

#[async_trait]
impl Notifier for TelegramAdapter {
    async fn notify_new_listing(
        &self,
        subscription: &Subscription,
        metadata: &ScrapeMetadata,
        listing: &Listing,
        channel_id: ChannelId,
    ) -> Result<(), String> {
        let chat_id = match channel_id {
            ChannelId::Telegram { chat_id } => chat_id,
            _ => return Err("Invalid channel ID: expected Telegram channel.".to_string()),
        };

        match self
            .bot
            .send_message(
                // temporary
                chat_id,
                self.format_notification(subscription, metadata, listing),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to send message: {e}")),
        }
    }

    async fn send_coconut(&self, channel_id: ChannelId) -> Result<(), String> {
        let chat_id = match channel_id {
            ChannelId::Telegram { chat_id } => chat_id,
            _ => return Err("Invalid channel ID: expected Telegram channel.".to_string()),
        };

        match self
            .bot
            .send_photo(chat_id, InputFile::file("coconut.jpg"))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to send message: {e}")),
        }
    }
}

// For the dispatcher.
pub async fn telegram_handler(
    bot: Bot,
    msg: Message,
    subscription_store: Arc<Mutex<SubscriptionStore>>,
    runtime_store: Arc<Mutex<RuntimeStateStore>>,
    monitor_manager: Arc<Mutex<MonitorManager>>,
    notifiers: NotifierRegistry,
) -> ResponseResult<()> {
    let handler = IncomingMessageHandler {
        subscriptions: subscription_store,
        monitor_manager,
    };
    let channel_id = msg.chat.id;
    let message = IncomingMessage::from_telegram(msg);

    let reply = handler.handle_message(message, runtime_store, notifiers);

    // We differentiate between errors and normal replies, but they are currently both handled
    // the same way.
    if let Err(e) = reply {
        bot.send_message(channel_id, e).await?;
    } else if let Ok(Some(reply)) = reply {
        bot.send_message(channel_id, reply).await?;
    }
    // Might need to refactor this.
    Ok(())
}
