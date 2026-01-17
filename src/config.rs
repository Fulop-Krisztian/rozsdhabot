use crate::integrations::{
    Controller, DiscordIntegration, NotifierRegistry, TelegramIntegration, TerminalIntegration,
};
use std::{collections::HashSet, sync::Arc};
use teloxide::Bot;

pub struct AppConfig {
    pub integrations: HashSet<Integration>,
    pub disable_saving: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Integration {
    Telegram { token: String },
    Discord { token: String },
    Terminal,
}

pub enum ConfigLoadMethod {
    File,
    Env,
}

impl AppConfig {
    pub fn get_config() -> Result<AppConfig, String> {
        // this is for testing only.
        let _ = dotenv::from_filename(".env");

        let mut integrations = HashSet::new();

        for integration in get_env_var("INTEGRATIONS")
            .expect("INTEGRATIONS not found in env")
            .clone()
            .split(',')
            .filter(|s| !s.is_empty())
        // filter dangling commas at the end of config
        // (eg. INTEGRATIONS=terminal,discord,)
        {
            match integration.to_uppercase().as_str() {
                "TELEGRAM" => {
                    let token = get_env_var("TELEGRAM_TOKEN")
                        .map_err(|_| "TELEGRAM_TOKEN not found in env")?;
                    integrations.insert(Integration::Telegram { token });
                }

                "DISCORD" => {
                    let token = get_env_var("DISCORD_TOKEN")
                        .map_err(|_| "DISCORD_TOKEN not found in env")?;
                    integrations.insert(Integration::Discord { token });
                }
                // This just prints what would be sent as messages to telegram as text to the
                // terminal. Mostly meant for debug purposes
                "TERMINAL" => {
                    integrations.insert(Integration::Terminal);
                }
                _ => {
                    tracing::error!(
                        "Invalid integration: '{}'. Check the INTEGRATIONS variable",
                        integration
                    );
                    // It's better if we exit here rather than continue.
                    std::process::exit(1);
                }
            }
        }

        let mut disable_saving = false;
        if get_env_var("DISABLE_SAVING").is_ok_and(|s| s == "true") {
            disable_saving = true;
        }

        Ok(AppConfig {
            integrations,
            disable_saving,
        })
    }

    pub fn get_integrations(config: &AppConfig) -> (Vec<Arc<dyn Controller>>, NotifierRegistry) {
        // This is just so that we don't run without any integrations.
        if config.integrations.is_empty() {
            panic!("No integrations found. Please add them to the .env file.");
        }

        let mut notifiers = NotifierRegistry::default();
        let mut controllers: Vec<Arc<dyn Controller>> = Vec::new();
        // The set ensures that there is only one instance of each integration.
        for integration in &config.integrations {
            match integration {
                Integration::Telegram { token } => {
                    let telegram_bot = Bot::new(token);
                    let integration = Arc::new(TelegramIntegration::new(telegram_bot));
                    // The unwrap here is safe because we just initalized the bot.
                    notifiers.telegram = Some(integration.clone());
                    controllers.push(integration);
                }

                Integration::Discord { token } => {
                    // same pattern
                    let discord_bot = DiscordIntegration::new(token);
                    // TODO:
                    notifiers.discord = None;
                }
                Integration::Terminal => {
                    notifiers.terminal = Some(Arc::new(TerminalIntegration::new()))
                }
            }
        }

        (controllers, notifiers)
    }
}

/// We don't care about the types of errors 'round these parts.
fn get_env_var(key: &str) -> Result<String, std::env::VarError> {
    // dotenv::var(key).map_err(|_| ())
    std::env::var(key)
}
