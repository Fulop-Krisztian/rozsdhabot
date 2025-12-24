// Dummy persistence for testing or for disabling persistence
mod dummy_persistence;
pub use dummy_persistence::DummyPersistence;

// file based persistence
mod file_persistence;
pub use file_persistence::FilePersistence;

// Store implementation for the runtime state
mod runtime_store;
pub use runtime_store::RuntimeStateStore;

// Store implementation for subscriptions
mod subscription_store;
pub use subscription_store::SubscriptionStore;

use crate::models::{Subscription, SubscriptionState};

pub trait Persistence: Send + Sync {
    fn load_subscriptions(&self) -> anyhow::Result<Vec<Subscription>>;
    fn save_subscriptions(&self, subscriptions: &[Subscription]) -> anyhow::Result<()>;

    fn load_states(&self) -> anyhow::Result<Vec<SubscriptionState>>;
    fn save_states(&self, states: &[SubscriptionState]) -> anyhow::Result<()>;
}
