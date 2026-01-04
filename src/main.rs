use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    config::AppConfig,
    integrations::NotifierRegistry,
    monitor::MonitorManager,
    storage::{
        DummyPersistence, FilePersistence, Persistence, RuntimeStateStore, SubscriptionStore,
    },
};
use tokio::time::sleep;
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod fetcher;
mod models;
mod monitor;

mod integrations;
mod parsers;
mod storage;

// TODO: kód összetartó kókusz
// TODO: a kókuszt ÉJÁJJÁL ellenőrzi
// https://www.youtube.com/watch?v=SmM0653YvXU
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // set up logging
    init_tracing();

    match std::fs::File::open("coconut.jpg") {
        Ok(_) => (),
        Err(_) => panic!("Nincs kokusz, nincs program."),
    }

    // Load the environment variables
    // this can crash if config is invalid
    let config = AppConfig::get_config().expect("Unable to load configuration");
    let (controllers, notifiers) = AppConfig::get_integrations(&config);

    const DATA_DIR: &str = "data";
    // Deciding what kind of persistence to use.
    let saver: Arc<dyn Persistence> = if config.disable_saving {
        tracing::warn!(
            "Nothing is saved to or loaded from disk. Disable in .env file if this is not intended."
        );
        Arc::new(DummyPersistence)
    } else {
        Arc::new(
            FilePersistence::new(DATA_DIR).expect("Failed to to initalize file based persistence"),
        )
    };

    let app_context = AppCtx {
        notifiers,
        ..AppCtx::with_persistence(saver)
    };

    // Each controller is launched in a separate task on startup.
    // let controllers: Vec<Arc<dyn Controller>>;
    //
    run_app(app_context, controllers).await;
    Ok(())
}

#[derive(Clone)]
/// Short for app context.
///
/// Stores the things needed to run the entire application.
pub struct AppCtx {
    /// Stores subscriptions, based upon which the monitors are started. Modified by the
    /// controllers.
    pub subscription_store: Arc<Mutex<SubscriptionStore>>,
    /// Stores the runtime state of monitors. This is modified by the monitors themselves.
    pub runtime_store: Arc<Mutex<RuntimeStateStore>>,
    /// Manages the monitors, state is modified by the controllers.
    pub monitor_manager: Arc<Mutex<MonitorManager>>,
    /// Passed along to each monitor for them to use. Immutable after startup.
    pub notifiers: NotifierRegistry,
}

use integrations::Controller;

impl AppCtx {
    /// Create a new context by fully specifying all fields.
    fn new(
        subscription_store: Arc<Mutex<SubscriptionStore>>,
        runtime_store: Arc<Mutex<RuntimeStateStore>>,
        monitor_manager: Arc<Mutex<MonitorManager>>,
        notifiers: NotifierRegistry,
    ) -> Self {
        Self {
            subscription_store,
            runtime_store,
            monitor_manager,
            notifiers,
        }
    }

    /// Create a new context with the given persistence.
    ///
    /// Other fields are set to default.
    fn with_persistence(persistence: Arc<dyn Persistence>) -> Self {
        Self::new(
            Arc::new(Mutex::new(SubscriptionStore::new(persistence.clone()))),
            Arc::new(Mutex::new(
                RuntimeStateStore::new(persistence.clone()).unwrap(),
            )),
            Arc::new(Mutex::new(MonitorManager::default())),
            NotifierRegistry::default(),
        )
    }
}

impl Default for AppCtx {
    /// Dummy persistence is used as default.
    fn default() -> Self {
        Self::new(
            Arc::new(Mutex::new(SubscriptionStore::new(Arc::new(
                DummyPersistence {},
            )))),
            Arc::new(Mutex::new(
                RuntimeStateStore::new(Arc::new(DummyPersistence {})).unwrap(),
            )),
            Arc::new(Mutex::new(MonitorManager::default())),
            NotifierRegistry::default(),
        )
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
    tracing::info!("🦀🦀🦀 Logging initialized, welcome to rozsdhabot! 🦀🦀🦀");
}

async fn run_app(context: AppCtx, controllers: Vec<Arc<dyn Controller>>) {
    // let mut handles = Vec::new();

    // Start monitors that were loaded from disk.
    const STAGGER: Duration = Duration::from_millis(1250);
    tracing::info!(
        "Starting all saved subscriptions with {}ms stagger. This may take a while...",
        STAGGER.as_millis()
    );

    for controller in controllers {
        let context = context.clone();
        tokio::spawn(async move {
            controller.start(context).await;
        });
    }

    // Persistently holding the mutex guard is fine as long as the handlers are launched
    // beforehand.
    //
    // This also prevents inconsistent state from the user sending a /del command before the given
    // monitor has started, since the mutex is held for the duration of startup, and nothing can be
    // modified.
    for sub in context
        .subscription_store
        .lock()
        .unwrap()
        .subscriptions
        .values()
        .cloned()
    {
        context.monitor_manager.lock().unwrap().start_monitor(
            sub,
            context.runtime_store.clone(),
            context.notifiers.clone(),
        );
        // Staggared startup to avoid rate limiting
        // The mutex is held for the duration of startup.
        sleep(STAGGER).await;
    }

    // SIGINT I think.
    tokio::signal::ctrl_c().await;
}
