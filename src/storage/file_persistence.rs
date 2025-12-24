// File based persistence for scraper data and runtime state.
use anyhow::Context;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    models::{Subscription, SubscriptionState},
    storage::Persistence,
};

impl Persistence for FilePersistence {
    fn load_subscriptions(&self) -> anyhow::Result<Vec<Subscription>> {
        load_json(&self.subscriptions_path)
    }

    fn save_subscriptions(&self, subs: &[Subscription]) -> anyhow::Result<()> {
        save_json(&self.subscriptions_path, subs)
    }

    fn load_states(&self) -> anyhow::Result<Vec<SubscriptionState>> {
        load_json(&self.state_path)
    }

    fn save_states(&self, states: &[SubscriptionState]) -> anyhow::Result<()> {
        save_json(&self.state_path, states)
    }
}

pub struct FilePersistence {
    subscriptions_path: PathBuf,
    state_path: PathBuf,
}

impl FilePersistence {
    pub fn new(data_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let data_dir = data_dir.as_ref();

        fs::create_dir_all(data_dir).context("Failed to create data directory")?;

        Ok(Self {
            subscriptions_path: data_dir.join("subscriptions.json"),
            state_path: data_dir.join("state.json"),
        })
    }
}

// We're doing async and multithreading so we need to do atomic writes.
fn atomic_write(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    let tmp_path = path.with_extension("tmp");

    fs::write(&tmp_path, content).with_context(|| format!("Failed to write to file {:?}", path))?;
    fs::rename(&tmp_path, path).with_context(|| format!("Failed to rename {:?}", path))?;

    Ok(())
}

fn load_json<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let bytes = fs::read(path).with_context(|| format!("Failed to read file {:?}", path))?;

    let data =
        serde_json::from_slice(&bytes).with_context(|| format!("Failed to parse {:?}", path))?;

    Ok(data)
}

fn save_json<T: serde::Serialize>(path: &Path, data: &[T]) -> anyhow::Result<()> {
    let bytes =
        serde_json::to_vec(data).with_context(|| format!("Failed to serialize {:?}", path))?;

    atomic_write(path, &bytes)
}
