
use crate::models::Listing;

pub trait Notifier {
    fn notify_new_listing(&self, listing: &Listing) -> Result<(), String>;

    fn process_message(&self, user: String, message_text: String){ 

    }
}

/// A notifier that prints the listing to the terminal. Intended for testing, but it's
/// fully functional.
pub struct TerminalNotifier;
impl TerminalNotifier {
    pub fn new() -> Self {
        Self {}
    }
}

impl Notifier for TerminalNotifier {
    fn notify_new_listing(&self, listing: &Listing) -> Result<(), String> {
        println!("Új hírdetés gecókám: {} {}", listing.title, listing.url);
        // TODO: prettier formatting. Right now it's just derived.
        println!("{listing:?}");
        // Can't really fail
        Ok(())
    }
}

pub struct TelegramNotifier {
    bot: teloxide::Bot,
}

impl TelegramNotifier {
    pub fn new(token: String) -> Self {
        TelegramNotifier {
            bot: teloxide::Bot::new(token),
        }
    }
}

use teloxide::{prelude, types::User}

impl Notifier for TelegramNotifier {



    fn notify_new_listing(&self, listing: &Listing) -> Result<(), String> {
        self.bot.copy_messages

        



    }
}
