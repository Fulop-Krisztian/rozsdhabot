use tokio::{sync::watch, task::JoinHandle};
use tracing::Instrument;

use crate::{
    fetcher::Fetcher, integrations::NotifierRegistry, models::Subscription,
    parsers::parse_hardverapro, storage::RuntimeStateStore,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// A monitor is responsible for running one subscription.
pub struct Monitor {
    subscription: Subscription,
    fetcher: Fetcher,
    runtime_store: Arc<Mutex<RuntimeStateStore>>,
    notifiers: NotifierRegistry,
}

#[derive(Default)]
pub struct MonitorManager {
    monitors: HashMap<u64, MonitorHandle>,
}

/// A monitor manager that is reponsible for spawning and keeping track of monitors.
impl MonitorManager {
    pub fn new(monitors: HashMap<u64, MonitorHandle>) -> Self {
        Self { monitors }
    }

    pub fn start_monitor(
        &mut self,
        subscription: Subscription,
        runtime_store: Arc<Mutex<RuntimeStateStore>>,
        notifiers: NotifierRegistry,
    ) {
        // We always want to see which monitor this is.
        let span = tracing::error_span!("monitor", subscription = subscription.id);

        let (shutdown_tx, shutdown_rx) = watch::channel(());

        let mut monitor = Monitor::new(notifiers, runtime_store, subscription);
        let id = monitor.subscription.id;

        let join = tokio::spawn(async move {
            monitor.run(shutdown_rx).instrument(span).await;
        });

        self.monitors.insert(
            id,
            MonitorHandle {
                shutdown: shutdown_tx,
                join,
            },
        );
    }

    pub fn stop_monitor(&mut self, id: u64) {
        tracing::debug!("Sending shutdown signal to monitor {}", id);
        if let Some(handle) = self.monitors.remove(&id) {
            // it might have already shut down if it encountered an error, in which case we don't
            // need to do anything
            let _ = handle.shutdown.send(());
        }
    }

    /// In the current implementation subscriptions are restarted when modified.
    pub fn restart_monitor(
        &mut self,
        subscription: Subscription,
        runtime_store: Arc<Mutex<RuntimeStateStore>>,
        notifiers: NotifierRegistry,
    ) {
        self.stop_monitor(subscription.id);
        self.start_monitor(subscription, runtime_store, notifiers);
    }
}

pub struct MonitorHandle {
    shutdown: watch::Sender<()>,
    join: JoinHandle<()>,
}

impl Monitor {
    pub fn new(
        notifiers: NotifierRegistry,
        runtime_store: Arc<Mutex<RuntimeStateStore>>,
        subscription: Subscription,
    ) -> Self {
        Self {
            // Contains the configuration for the subscription.
            subscription,
            runtime_store,
            // A new fetcher that is inconsistent across different runs. This is on purpose.
            fetcher: Fetcher::new(),
            notifiers,
        }
    }

    pub async fn run(&mut self, mut shutdown: watch::Receiver<()>) {
        tracing::info!("Starting monitor");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            self.subscription.config.interval,
        ));

        loop {
            tokio::select! {
                _ = shutdown.changed() => {tracing::info!("Monitor stopping"); break},
                // errors are ignored here.
                _ = interval.tick() => self.scrape().await.unwrap_or(()),
            }
        }
    }

    /// This is where the magic happens.
    async fn scrape(&self) -> Result<(), ()> {
        // Telegram is pretty lenient with rate limiting
        const NOTIFY_STAGGER: std::time::Duration = std::time::Duration::from_millis(100);

        tracing::debug!("scraping...");

        let body = match self.fetcher.fetch(&self.subscription.url).await {
            Ok(body) => body,
            Err(e) => {
                tracing::error!("Failed to fetch site: {e}. Possibly invalid URL.");
                return Err(());
            }
        };

        // Where the parser is run.
        // Future expansion: We could run a parser based on the type of subscription (implementing
        // multiple parsers).
        let page = parse_hardverapro(&body);

        // logging the results
        if page.listings.is_empty() {
            if page.failures.is_empty() {
                tracing::warn!("No listings found on page at all. URL to scrape may be incorrect");
            } else {
                tracing::warn!(
                    "Failed to parse any listings. {} skips, and {} unparsable fields.",
                    page.skipped_listings_count(),
                    page.unparsable_listing_count()
                );
            }
        } else {
            tracing::info!(
                "Scraped {} listings, skipped {}",
                page.listings.len(),
                page.skipped_listings_count()
            );

            if page.unparsable_listing_count() > 0 {
                for failure in &page.failures {
                    tracing::warn!("Failed to parse {}: {:?}", failure.field, failure.value);
                }
            }
        }

        let state = self
            .runtime_store
            .lock()
            .unwrap()
            .get(self.subscription.id)
            .cloned();

        match state {
            Some(..) => {}
            // If we didn't find an entry for our subscription, we create one.
            None => {
                let newest = page.listings.iter().map(|l| l.id).max();
                if let Some(id) = newest {
                    self.runtime_store
                        .lock()
                        .unwrap()
                        .update_last_seen(self.subscription.id, id)
                        .unwrap();
                }
                // TODO: This is incorrect logic for what I want to do. This doesn't notify until
                // it has seen a new listing. I only want it to skip notifying on the first run.
                tracing::info!("no listing seen before: notifications will not be sent");
                return Ok(());
            }
        };

        // unwrap is safe because we just checked if it was Some
        let last_seen = match state.unwrap().last_seen {
            Some(last_seen) => last_seen,
            // If there was no last seen, we update it.
            None => {
                let newest = page.listings.iter().map(|l| l.id).max();
                if let Some(id) = newest {
                    self.runtime_store
                        .lock()
                        .unwrap()
                        .update_last_seen(self.subscription.id, id)
                        .unwrap();
                }
                tracing::info!("no listing seen before: notifications will not be sent");
                return Ok(());
            }
        };

        for channel in &self.subscription.channels {
            let Some(notifier) = self.notifiers.notifier_for(channel) else {
                tracing::warn!(
                    "No notifier for channel: {channel:?}. You should enable the integration for it."
                );
                continue;
            };

            let channel = *channel;

            for listing in page.listings.iter().filter(|l| l.id > last_seen) {
                notifier
                    .notify_new_listing(&self.subscription, &page.metadata, listing, channel)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to notify for listing {listing:?}: {e}");
                    })?;
                tokio::time::sleep(NOTIFY_STAGGER).await;
            }
        }

        if let Some(new_last_seen) = page.listings.iter().map(|l| l.id).max() {
            self.runtime_store
                .lock()
                .unwrap()
                .update_last_seen(self.subscription.id, new_last_seen)
                .unwrap();
        }

        Ok(())
    }

    // async fn filter_new_listings(&self, page: Page) -> Page {}

    // async fn notify(&self, page: Page) {}
}
