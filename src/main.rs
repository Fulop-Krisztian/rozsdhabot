use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    config::{Config, Integration, get_config},
    integrations::{NotifierRegistry, TelegramAdapter, TerminalAdapter},
    monitor::MonitorManager,
    storage::{
        DummyPersistence, FilePersistence, Persistence, RuntimeStateStore, SubscriptionStore,
    },
};
use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    prelude::Dispatcher,
    types::{Message, Update},
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
    const DATA_DIR: &str = "data";

    match std::fs::File::open("coconut.jpg") {
        Ok(_) => (),
        Err(_) => panic!("Nincs kokusz, nincs program."),
    }

    // set up logging
    init_tracing();

    // Load the environment variables
    // this can crash if config is invalid
    let config = get_config().expect("Unable to load configuration");
    if config.integrations.is_empty() {
        panic!("No integrations found. Please add them to the .env file.");
    }

    let saver: Arc<dyn Persistence> = if config.disable_saving {
        tracing::warn!(
            "Nothing is saved to or loaded from disk. Disable in .env file if this is not intended."
        );
        Arc::new(DummyPersistence)
    } else {
        Arc::new(FilePersistence::new(DATA_DIR).unwrap())
    };

    let substore = Arc::new(Mutex::new(SubscriptionStore::new(saver.clone())));
    let runtime_state = Arc::new(Mutex::new(RuntimeStateStore::new(saver.clone()).unwrap()));
    let monitor_manager = Arc::new(Mutex::new(MonitorManager::new()));

    run_app(config, substore, runtime_state, monitor_manager).await;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
    tracing::info!("🦀🦀🦀 Logging initialized, welcome to rozsdhabot! 🦀🦀🦀");
}

async fn run_app(
    config: Config,
    subscription_store: Arc<Mutex<SubscriptionStore>>,
    state_store: Arc<Mutex<RuntimeStateStore>>,
    monitor_manager: Arc<Mutex<MonitorManager>>,
) {
    let mut handles = Vec::new();
    let mut notifiers = NotifierRegistry::default();

    let mut telegram_bot: Option<Bot> = None;

    // The set ensures that there is only one instance of each integration.
    for integration in &config.integrations {
        match integration {
            Integration::Telegram { token } => {
                telegram_bot = Some(Bot::new(token));
                // The unwrap here is safe because we just initalized the bot.
                notifiers.telegram = Some(Arc::new(TelegramAdapter::new(
                    telegram_bot.clone().unwrap(),
                )));
            }

            Integration::Discord { token } => {
                notifiers.discord = None;
                // same pattern
            }
            Integration::Terminal => notifiers.terminal = Some(Arc::new(TerminalAdapter::new())),
        }
    }

    // Start monitors that were loaded from disk.
    const STAGGER: Duration = Duration::from_millis(1250);
    tracing::info!(
        "Starting all saved subscriptions with {}ms stagger. This may take a while...",
        STAGGER.as_millis()
    );
    for sub in subscription_store
        .lock()
        .unwrap()
        .subscriptions
        .values()
        .cloned()
    {
        monitor_manager
            .lock()
            .unwrap()
            .start_monitor(sub, state_store.clone(), notifiers.clone());
        // Staggared startup to avoid rate limiting
        // The mutex is held for the duration of startup.
        sleep(STAGGER).await;
    }

    if let Some(telegram_bot) = telegram_bot {
        handles.push(tokio::spawn(run_telegram_dispatcher(
            telegram_bot,
            subscription_store.clone(),
            state_store.clone(),
            monitor_manager.clone(),
            notifiers,
        )));
    }

    // SIGINT I think.
    tokio::signal::ctrl_c().await;
}

pub async fn run_telegram_dispatcher(
    bot: Bot,
    subscription_store: Arc<Mutex<SubscriptionStore>>,
    runtime_store: Arc<Mutex<RuntimeStateStore>>,
    monitor_manager: Arc<Mutex<MonitorManager>>,
    notifiers: NotifierRegistry,
) {
    use integrations::telegram_handler;
    let handler = Update::filter_message().endpoint(move |bot: Bot, msg: Message| {
        let subscription_store = subscription_store.clone();
        let runtime_store = runtime_store.clone();
        let monitor_manager = monitor_manager.clone();
        let notifiers = notifiers.clone();
        async move {
            telegram_handler(
                bot,
                msg,
                subscription_store,
                runtime_store,
                monitor_manager,
                notifiers,
            )
            .await
        }
    });

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
