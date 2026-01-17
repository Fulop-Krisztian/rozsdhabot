use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;

pub fn register() -> CreateCommand {
    CreateCommand::new("add")
        .description("Add a new subscription with the given URL")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "The URL to scrape")
                .required(true),
        )
}
