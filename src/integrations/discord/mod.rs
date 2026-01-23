mod commands;

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    AppCtx,
    integrations::{
        Controller, Notifier,
        message_handler::{IncomingMessage, handle_message},
    },
    models::{ChannelId, Listing, Subscription},
    parsers::ScrapeMetadata,
};
use serenity::{
    Client,
    all::{Context, EventHandler, GatewayIntents, Http},
};

pub struct DiscordController {
    token: String,
}

impl DiscordController {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
}

#[async_trait]
impl Controller for DiscordController {
    async fn start(self: Box<Self>, context: AppCtx) -> () {
        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(self.token, intents)
            .event_handler(Handler)
            .await
            .expect("Failed to initalize Discord client");

        {
            let mut data = client.data.write().await;
            data.insert::<AppCtxKey>(context);
        }

        client.start().await;
    }
}

pub struct DiscordNotifier {
    http: Arc<Http>,
}

impl DiscordNotifier {
    pub fn new(token: &str) -> Self {
        Self {
            http: Arc::new(serenity::all::Http::new(token)),
        }
    }

    fn format_notification(
        &self,
        sub: &Subscription,
        metadata: &ScrapeMetadata,
        listing: &Listing,
    ) -> String {
        use teloxide::utils::markdown;
        let price = listing.price;
        let link = markdown::link(&listing.url, markdown::escape(&listing.title).as_str());
        let sub_title =
            markdown::escape(sub.name.clone().unwrap_or("(unnamed)".to_string()).as_str());

        // Discord supports multiple embeds, which is not supported in Telegram and we relied on it
        // for a nicer looking notification. This has less functionality but is nicer to look at.
        // let sub_title = markdown::link(
        //     &sub.url,
        //     markdown::escape(sub.name.clone().unwrap_or("(unnamed)".to_string()).as_str()).as_str(),
        // );
        //
        let id = sub.id.to_string();
        let seller_name = markdown::escape(listing.seller_name.as_str());
        let seller_ratings = markdown::escape(listing.seller_ratings.to_string().as_str());
        let cities = markdown::escape(listing.cities.join(", ").as_str());

        let pricerange = markdown::escape(
            match (metadata.min_price, metadata.max_price) {
                (Some(min), Some(max)) => format!("-# between {min:.0} - {max:.0} Ft"),
                (Some(min), None) => format!("-# above {min:.0} Ft"),
                (None, Some(max)) => format!("-# under {max:.0} Ft"),
                (None, None) => "".to_string(),
            }
            .as_str(),
        );

        format!(
            "
## New: {link}
## {price} Ft
{cities}
\\- {seller_name} \\(\\+{seller_ratings}\\)

-# From subscription:
-# {sub_title} \\({id}\\):
{pricerange}
",
        )
    }
}

#[async_trait]
impl Notifier for DiscordNotifier {
    async fn notify_new_listing(
        &self,
        subscription: &Subscription,
        metadata: &ScrapeMetadata,
        listing: &Listing,
        channel_id: ChannelId,
    ) -> Result<(), String> {
        let channel = match channel_id {
            ChannelId::Discord { channel } => channel,
            _ => {
                return Err(format!(
                    "Invalid channel ID: {:?} expected Discord channel.",
                    channel_id
                )
                .to_string());
            }
        };

        channel
            .say(
                self.http.clone(),
                self.format_notification(subscription, metadata, listing),
            )
            .await
            .map_err(|e| e.to_string())
            .map(|_| ())
    }
    async fn send_coconut(&self, channel_id: ChannelId) -> Result<(), String> {
        todo!()
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // async fn message(&self, ctx: Context, msg: Message) {}

    // TODO: this could be a future way of handling messages that is better integrated with discord.
    //
    // async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    //     if let Interaction::Command(command) = interaction {
    //         match command.data.name.as_str() {
    //             "add" => {
    //                 // take the first option
    //                 let content = command.data.options.iter().next().unwrap().value.as_str();
    //                 message_handler::add_subscription(
    //                     IncomingMessage::from_discord_command(command, content),
    //                     self.context,
    //                 );
    //             }
    //             "start" => {
    //                 // take the first option
    //                 let param = command.data.options.iter().next().unwrap().value.as_str();
    //             }
    //         }
    //
    //         // We just pass the whole thing as it were typed in as a command to the command
    //         // handler.
    //         // let command = command.data.name + " " + &command.data.options.join(" ");
    //
    //         // command.data.name.as_str();
    //         todo!()
    //     }
    // }

    async fn message(&self, ctx: Context, msg: serenity::all::Message) {
        // NOTE:
        // It seems that the message event is triggered for every message, even if we are the ones
        // that sent it. This could result in an infinite loop, so watch out.
        tracing::debug!("discord: message: {:?}", msg);

        // Ignore messages from ourselves.
        if msg.author.id == ctx.cache.current_user().id {
            return;
        };

        // NOTE:
        // Ignore messages from bots. This could be optional.
        if msg.author.bot {
            return;
        }

        let app_ctx = {
            let data = ctx.data.read().await;
            data.get::<AppCtxKey>()
                .expect("AppCtx not initalized")
                .clone()
        };

        let channel = msg.channel_id;
        let message = IncomingMessage::from_discord_message(msg);

        let reply = handle_message(message, app_ctx);

        // First we're unwrapping the result of message processing, and then we're unwrapping the result of the
        // message sending. TODO: refactor.
        if let Err(e) = reply {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            if let Err(why) = channel.say(&ctx.http, e).await {
                tracing::error!("Error sending message: {why:?}");
            }
        } else if let Ok(Some(reply)) = reply {
            if let Err(why) = channel.say(&ctx.http, reply).await {
                tracing::error!("Error sending message: {why:?}");
            }
        }
    }

    async fn ready(&self, _: serenity::all::Context, ready: serenity::all::Ready) {
        tracing::info!(
            "discord integration ready: {} is connected",
            ready.user.name
        );
    }
}

struct AppCtxKey;
impl serenity::prelude::TypeMapKey for AppCtxKey {
    type Value = AppCtx;
}
