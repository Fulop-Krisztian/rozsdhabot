use crate::models::ListingId;
use std::{collections::HashMap, sync::Arc};

use crate::{models::SubscriptionState, storage::Persistence};

/// This file stores the runtime state of the bot.
pub struct RuntimeStateStore {
    states: HashMap<u64, SubscriptionState>, // sub id => state
    persistence: Arc<dyn Persistence>,
}

impl RuntimeStateStore {
    pub fn new(persistence: Arc<dyn Persistence>) -> anyhow::Result<Self> {
        let states_vec = persistence.load_states()?;
        let states = states_vec
            .into_iter()
            .map(|s| (s.subscription_id, s))
            .collect();

        Ok(Self {
            states,
            persistence,
        })
    }

    pub fn get(&self, id: u64) -> Option<&SubscriptionState> {
        self.states.get(&id)
    }
    #[tracing::instrument(name = "RuntimeStateStore::update_last_seen", skip(self))]
    pub fn update_last_seen(&mut self, id: u64, listing_id: ListingId) -> anyhow::Result<()> {
        let entry = self.states.entry(id).or_insert(SubscriptionState {
            subscription_id: id,
            last_seen: None,
        });

        entry.last_seen = Some(listing_id);
        tracing::trace!("Updated last seen for subscription {}", id);
        self.persistence
            .save_states(&self.states.values().cloned().collect::<Vec<_>>())?;

        Ok(())
    }

    #[tracing::instrument(name = "RuntimeStateStore::remove", skip(self))]
    pub fn remove(&mut self, id: u64) {
        self.states.remove(&id);
        tracing::trace!("Removed subscription state for subscription {}", id);
        self.persistence
            .save_states(&self.states.values().cloned().collect::<Vec<_>>())
            .unwrap();
    }
}
