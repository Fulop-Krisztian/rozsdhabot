use std::{collections::HashMap, sync::Arc};

use url::Url;

use crate::{
    models::{ChannelId, OwnerId, Subscription, SubscriptionConfig},
    storage::Persistence,
};

/// A store for subscriptions. Abstracts the storage method we use for subscriptions.
pub struct SubscriptionStore {
    pub subscriptions: HashMap<u64, Subscription>,
    persistence: Arc<dyn Persistence>,
    next_id: u64,
}

impl SubscriptionStore {
    // TODO: load from file
    pub fn new(persistence: Arc<dyn Persistence>) -> Self {
        let mut subscriptions = HashMap::new();
        let mut next_id = 1;
        for sub in persistence.load_subscriptions().unwrap_or_default() {
            // By figuring out the highest id while we are loading we avoid having to save it.
            if sub.id >= next_id {
                next_id = sub.id + 1;
            }
            subscriptions.insert(sub.id, sub);
        }

        Self {
            subscriptions,
            persistence,
            // We start at 1 because this is visible to users.
            next_id,
        }
    }

    fn get_name_from_url(url: &str) -> Option<String> {
        let url = Url::parse(url).ok()?;

        // 1. Prefer explicit query parameter
        if let Some(value) = url
            .query_pairs()
            .find(|(k, v)| k == "stext" && !v.is_empty())
            .map(|(_, v)| v.into_owned())
        {
            return Some(value);
        }

        // 2. Fallback to strictest category
        Self::category_from_path(&url)
    }

    fn category_from_path(url: &Url) -> Option<String> {
        let segments = url.path_segments()?;

        // Collect because we need to inspect relative position
        let parts: Vec<_> = segments.collect();

        match parts.as_slice() {
            // .../<category>/something
            // usually: .../<category>/keres.php
            [.., category, _] => Some(category.to_string()),
            _ => None,
        }
    }

    pub fn add_subscription(&mut self, url: String, channel: ChannelId, owner: OwnerId) -> u64 {
        let name = Self::get_name_from_url(&url);

        let subscription = Subscription {
            id: self.next_id,
            name,
            channels: vec![channel], // Only the channel this is called from is added at first.
            owner,
            url,
            config: SubscriptionConfig::default(),
            // Maybe in the future.
            // platform_config: ChannelConfig::Telegram { thread_id: None },
            // metrics: SubscriptionMetrics::new(),
            created_at: chrono::Local::now().naive_local(),
        };
        self.subscriptions.insert(subscription.id, subscription);
        self.next_id += 1;

        self.persistence
            .save_subscriptions(&self.subscriptions.values().cloned().collect::<Vec<_>>());

        self.next_id - 1
    }

    pub fn remove_subscription(&mut self, id: u64) -> bool {
        let removed = self.subscriptions.remove(&id).is_some();
        self.persistence
            .save_subscriptions(&self.subscriptions.values().cloned().collect::<Vec<_>>());
        removed
    }

    pub fn get_subscription(&self, id: u64) -> Option<&Subscription> {
        self.subscriptions.get(&id)
    }

    /// Returns a list of references to all subscriptions in a channel.
    pub fn list_by_channel(&self, channel: ChannelId) -> Vec<&Subscription> {
        // PERF: This is a linear search through subscriptions.
        // We could implement a hasmap for this too that is managed alongside the subscriptions,
        // but it's fine for now.
        self.subscriptions
            .values()
            .filter(|s| s.channels.contains(&channel))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::DummyPersistence;

    use super::*;

    #[test]
    fn test_subscription_store() {
        let mut store = SubscriptionStore::new(Arc::new(DummyPersistence {}));
        // We can susccessfully add
        store.add_subscription(
            "https://hardverapro.hu".to_string(),
            ChannelId::Telegram {
                chat_id: teloxide::types::ChatId(1),
            },
            OwnerId::Telegram {
                user_id: Some(teloxide::types::UserId(1)),
            },
        );

        let subscription = Subscription {
            id: 1,
            name: Some("test".to_string()),
            channels: vec![ChannelId::Telegram {
                chat_id: teloxide::types::ChatId(1),
            }],
            owner: OwnerId::Telegram {
                user_id: Some(teloxide::types::UserId(1)),
            },
            url: "https://hardverapro.hu".to_string(),
            config: SubscriptionConfig::default(),
            // platform_config: ChannelConfig::Telegram { thread_id: None },
            // metrics: SubscriptionMetrics::new(),
            // This is needed to compare the creation time.
            created_at: store.get_subscription(1).unwrap().created_at,
        };

        // We can retrieve it normally
        assert_eq!(store.get_subscription(1), Some(&subscription));

        // We can retrieve it with by the channel id.
        assert_eq!(
            store.list_by_channel(ChannelId::Telegram {
                chat_id: teloxide::types::ChatId(1)
            }),
            vec![&subscription]
        );

        store.remove_subscription(1);
        // We can remove it.
        assert_eq!(store.get_subscription(1), None);
    }
}
