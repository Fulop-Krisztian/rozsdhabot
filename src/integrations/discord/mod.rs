mod commands;

use crate::{
    AppCtx,
    integrations::{Controller, Notifier},
    models::{ChannelId, Listing, Subscription},
    parsers::ScrapeMetadata,
};

pub struct DiscordIntegration {
    pub client: serenity::Client,
}

impl DiscordIntegration {
    pub fn new(token: &str) -> Self {
        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES;

        let client = Client::builder(token, intents);

        todo!()
    }
}

use async_trait::async_trait;
use serenity::{
    Client,
    all::{Context, EventHandler, GatewayIntents, Interaction},
};
#[async_trait]
impl Notifier for DiscordIntegration {
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

#[async_trait]
impl Controller for DiscordIntegration {
    async fn start(&self, context: AppCtx) -> () {
        todo!()
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // We just pass the whole thing as it were typed in as a command to the command
            // handler.
            // let command = command.data.name + " " + &command.data.options.join(" ");

            // command.data.name.as_str();
            todo!()
        }
    }
}
