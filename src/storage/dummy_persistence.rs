use crate::{
    models::{Subscription, SubscriptionState},
    storage::Persistence,
};

pub struct DummyPersistence;

impl Persistence for DummyPersistence {
    fn load_subscriptions(&self) -> anyhow::Result<Vec<Subscription>> {
        // println!("Loaded subscriptions");
        Ok(Vec::new())
    }

    fn save_subscriptions(&self, subs: &[Subscription]) -> anyhow::Result<()> {
        // println!("Saved subscriptions: {subs:?}");
        Ok(())
    }

    fn load_states(&self) -> anyhow::Result<Vec<SubscriptionState>> {
        // println!("Loaded states");
        Ok(Vec::new())
    }

    fn save_states(&self, states: &[SubscriptionState]) -> anyhow::Result<()> {
        // println!("Saved states: {states:?}");
        Ok(())
    }
}
