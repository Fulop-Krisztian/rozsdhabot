use crate::{
    AppCtx,
    models::{ChannelId, OwnerId},
};

/// An representation of an incoming message that is universal for all adapters.
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    /// Should go mostly unused. This is the ID of the message. Unique what it represents per
    /// platform.
    pub message_id: i64,
    /// Use this to identify where to send the reply, or to store the subscription.
    pub channel_id: ChannelId,
    pub sender: OwnerId,
    pub content: String,
}

/// Always returns a string that represents a reply to the message. This message should then be sent
/// to the platform.
///
/// This function modifies the context via interior mutability (through Mutexes).
pub fn handle_message(message: IncomingMessage, context: AppCtx) -> Result<Option<String>, String> {
    let command = message.content.split_whitespace().next().unwrap_or("");

    // Nothing needs to be done if the message is not a command.
    if !(command.starts_with('/')) {
        return Ok(None);
    }

    const HELP_MESSAGE: &str = "/help                   | Show this help message.
/add URL                | Add a new subscription. The URL is not checked for validity.
/del ID                 | Delete a subscription.
/list                   | List all subscriptions for the current channel.
/info ID                | Show metrics for a subscription.

variables:
ID: The subscription ID. You can get this by using /list.
URL: The URL to scrape. Only hardverapro is supported currently.
";

    const START_MESSAGE: &str = "Hello. This is rozsdhabot. Type /help for the list of commands.";

    // TODO: Handle mutliple additions, infos, removals, etc at the same time with a single
    // command (e.g. /add url1 url2 url3)
    match command {
        "/start" => Ok(Some(START_MESSAGE.to_string())),

        "/add" => add_subscription(message, context),

        "/del" => delete_subscription(message, context),

        "/list" => list_channel_subs(message, context),
        "/ls" => list_channel_subs(message, context),

        "/info" => sub_details(message, context), // set the name of a subscription. This will need an ID and the rest of the string will
        // be the name.
        // "/settings" => {}
        // "/seturl" => {}
        // "/setinterval" => {}
        //
        "/help" => Ok(Some(HELP_MESSAGE.to_string())),

        // Unrecognized commands warrant no reply.
        _ => Ok(None),
    }
}

pub fn add_subscription(
    message: IncomingMessage,
    context: AppCtx,
) -> Result<Option<String>, String> {
    let url = message
        .content
        .strip_prefix("/add")
        .unwrap()
        .trim()
        .to_string();

    if url.is_empty() {
        return Ok(Some(
            "Could not find URL in message. Usage: /add URL".to_string(),
        ));
    }

    let id = context.subscription_store.lock().unwrap().add_subscription(
        url.clone(),
        message.channel_id,
        message.sender,
    );

    let sub = context
        .subscription_store
        .lock()
        .unwrap()
        .get_subscription(id)
        .unwrap()
        .clone();

    context.monitor_manager.lock().unwrap().start_monitor(
        sub,
        context.runtime_store,
        context.notifiers,
    );

    Ok(Some(format!("New subscription added with ID: {}", id)))
}

/// A bit unsecure because subscriptions can be deleted from channels that they don't
/// belong to.
pub fn delete_subscription(
    message: IncomingMessage,
    context: AppCtx,
) -> Result<Option<String>, String> {
    let ids: Vec<u64> = message
        .content
        .strip_prefix("/del")
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<u64>())
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Could not parse ID: {}", e))?;

    let mut removed: Vec<u64> = Vec::new();
    let mut not_removed: Vec<u64> = Vec::new();

    // We can assume that the IDs are valid integers, though they might not refer to existing
    // subscriptions.
    for id in ids {
        context.monitor_manager.lock().unwrap().stop_monitor(id);
        context.runtime_store.lock().unwrap().remove(id);
        match context
            .subscription_store
            .lock()
            .unwrap()
            .remove_subscription_channel(id, message.channel_id)
        {
            true => removed.push(id),
            // This is not considered an error.
            false => not_removed.push(id),
        }
    }

    let removeds = removed
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let not_removeds = not_removed
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let mut buffer = String::new();

    if !removed.is_empty() {
        buffer += format!("Removed subscription with ID: {}\n", removeds).as_str();
    }

    if !not_removed.is_empty() {
        buffer += &format!(
            "Subscription {} was not removed because it doesn't exist in this channel\n",
            not_removeds
        );
    }

    Ok(Some(buffer))
}

/// Current channel only.
fn list_channel_subs(message: IncomingMessage, context: AppCtx) -> Result<Option<String>, String> {
    let reply: String = context
        .subscription_store
        .lock()
        .unwrap()
        .list_by_channel(message.channel_id)
        .clone()
        .iter()
        // TODO: improve formatting for this message
        .map(|sub| {
            if sub.name.is_some() {
                format!(
                    "ID:\t{}\t({}): {}\n",
                    sub.id,
                    sub.created_at.format("%Y-%m-%d %H:%M"),
                    sub.name.clone().unwrap(),
                )
            } else {
                format!(
                    "ID:\t{}\t({})\n",
                    sub.id,
                    sub.created_at.format("%Y-%m-%d %H:%M")
                )
            }
            // format!(
            //     "{:?} {:?}",
            //     sub.id,
            //     runtime_store.lock().unwrap().get(sub.id)
            // )
        })
        .collect();

    if reply.is_empty() {
        Ok(Some("No subscriptions found".to_string()))
    } else {
        Ok(Some(reply))
    }
}

pub fn sub_details(message: IncomingMessage, context: AppCtx) -> Result<Option<String>, String> {
    let id = message
        .content
        .strip_prefix("/info")
        .unwrap()
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Could not parse ID: {}", e))?;

    match context
        .subscription_store
        .lock()
        .unwrap()
        .get_subscription(id)
    {
        Some(sub) => Ok(Some(format!("{:?}", sub))),
        None => Ok(Some(format!("Subscription with ID {} does not extist", id))),
    }
}

impl IncomingMessage {
    pub fn from_telegram(message: teloxide::types::Message) -> Self {
        Self {
            message_id: message.id.0 as i64,
            channel_id: ChannelId::Telegram {
                chat_id: message.chat.id,
            },
            sender: OwnerId::Telegram {
                // PERF: A clone here is not ideal but a quick fix for now.
                user_id: message.from.clone().map(|u| u.id),
            },
            // TODO: review if this is correct.
            // Right now we assume that if a message's text method is None, it's the same as and
            // empty string.
            //
            // NOTE: It's probably good for our purposes, since we don't need to handle cases where
            // we get anything other than a text message.
            content: message.text().unwrap_or("").to_string(),
        }
    }

    pub fn from_discord_command() {
        todo!()
    }
}

// TODO: maybe we could do impl From for IncomingMessage
