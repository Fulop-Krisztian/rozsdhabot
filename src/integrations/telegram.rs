use crate::{
    AppCtx,
    integrations::{
        Controller, Notifier,
        message_handler::{IncomingMessage, handle_message},
    },
    models::{ChannelId, Listing, Subscription},
    parsers::ScrapeMetadata,
};

use async_trait::async_trait;
use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    payloads::SendMessageSetters,
    prelude::{Dispatcher, Requester, ResponseResult},
    types::{InputFile, Message, Update},
};

pub struct TelegramIntegration {
    pub bot: teloxide::Bot,
}

impl TelegramIntegration {
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
impl Notifier for TelegramIntegration {
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

#[async_trait]
impl Controller for TelegramIntegration {
    /// Modifies the AppCtx by adding itself as a controller.
    async fn start(&self, context: AppCtx) -> () {
        let handler = Update::filter_message().endpoint(move |bot: Bot, msg: Message| {
            let context = context.clone();
            async { telegram_handler(bot, msg, context).await }
        });
        Dispatcher::builder(self.bot.clone(), handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}

// For the dispatcher.
pub async fn telegram_handler(bot: Bot, msg: Message, context: AppCtx) -> ResponseResult<()> {
    let channel_id = msg.chat.id;
    let message = IncomingMessage::from_telegram(msg);

    let reply = handle_message(message, context);

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
