use std::collections::HashSet;

pub struct Config {
    pub integrations: HashSet<Integration>,
    pub disable_saving: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Integration {
    Telegram { token: String },
    Discord { token: String },
    Terminal,
}

/// We don't care about the types of errors 'round these parts.
fn get_env_var(key: &str) -> Result<String, std::env::VarError> {
    // dotenv::var(key).map_err(|_| ())
    std::env::var(key)
}

pub fn get_config() -> Result<Config, String> {
    // dotenv::from_filename(".env").map_err(|_| "Couldn't find .env file")?;
    let mut integrations = HashSet::new();

    for integration in get_env_var("INTEGRATIONS")
        .expect("INTEGRATIONS not found in env")
        .clone()
        .to_uppercase()
        .split(',')
        .filter(|s| !s.is_empty())
    // filter dangling commas at the end of config
    // (eg. INTEGRATIONS=terminal,discord,)
    {
        match integration {
            "TELEGRAM" => {
                let token =
                    get_env_var("TELEGRAM_TOKEN").map_err(|_| "TELEGRAM_TOKEN not found in env")?;
                integrations.insert(Integration::Telegram { token });
            }

            "DISCORD" => {
                let token =
                    get_env_var("DISCORD_TOKEN").map_err(|_| "DISCORD_TOKEN not found in env")?;
                integrations.insert(Integration::Discord { token });
            }
            // This just prints what would be sent as messages to telegram as text to the
            // terminal. Mostly meant for debug purposes
            "TERMINAL" => {
                integrations.insert(Integration::Terminal);
            }
            _ => println!("Invalid integration: {}. Ignoring", integration),
        }
    }

    let mut disable_saving = false;
    if get_env_var("DISABLE_SAVING").is_ok_and(|s| s == "true") {
        disable_saving = true;
    }

    Ok(Config {
        integrations,
        disable_saving,
    })
}
