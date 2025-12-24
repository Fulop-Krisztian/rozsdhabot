use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::listing::ListingId;

/// Contains the information needed to identify a channel. Differs for different platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelId {
    // This is always just stdout.
    Terminal,
    Telegram { chat_id: teloxide::types::ChatId },
    // GuildId is the group, channel_id is the channel.
    Discord { guild_id: i64, channel_id: i64 },
}

// /// Might be expanded in the future.
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub enum ChannelConfig {
//     Telegram { thread_id: Option<i32> },
//     Discord {},
// }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionConfig {
    /// How often we should check the site.
    pub interval: u64,

    // These are hardverapro specific.
    pub show_bazar: bool,
    pub show_featured: bool,
    pub show_regular: bool,
}

impl SubscriptionConfig {
    pub fn default() -> Self {
        Self {
            interval: 60,
            show_bazar: false,
            show_featured: true,
            show_regular: true,
        }
    }
}

/// Different types of owners for different adapters.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OwnerId {
    Telegram {
        user_id: Option<teloxide::types::UserId>,
    },
    Discord {
        user_id: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    /// The id of the subscription. Can be thought of as a primary key.
    pub id: u64,

    /// We do our best to generate a name from the URL, but it can't be guaranteed.
    pub name: Option<String>,

    pub channels: Vec<ChannelId>,
    /// The user that owns the subscription.
    /// for a future permission system.
    pub owner: OwnerId,

    /// What we are scraping.
    pub url: String,

    /// Subscription configuration.
    pub config: SubscriptionConfig,

    /// Platform specific configuration.
    // pub platform_config: ChannelConfig,

    /// Light telemetry
    // pub metrics: SubscriptionMetrics,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionState {
    pub subscription_id: u64,
    pub last_seen: Option<ListingId>,
}
