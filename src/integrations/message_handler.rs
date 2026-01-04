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

    // Nothing needs to be doen if the message is not a command.
    if !(command.starts_with('/')) {
        return Ok(None);
    }

    // TODO: Handle mutliple additions, infos, removals, etc at the same time with a single
    // command (e.g. /add url1 url2 url3)

    // This is only safe to call if the message actually starts with the command
    // This is where messages are interpreted.
    match command {
        "/start" => Ok(Some(
            "Hello. This is rozsdhabot. Type /help for the list of commands.".to_string(),
        )),
        "/add" => {
            let url = message
                .content
                .strip_prefix(command)
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

        // A bit unsecure because subscriptions can be deleted from channels that they don't
        // belong to.
        "/del" => {
            let id = message
                .content
                .strip_prefix(command)
                .unwrap()
                .trim()
                .parse::<u64>()
                .map_err(|e| format!("Could not parse ID: {}", e))?;

            context.monitor_manager.lock().unwrap().stop_monitor(id);

            // This is when we want to remove a subscription from the runtime store.
            context.runtime_store.lock().unwrap().remove(id);

            match context
                .subscription_store
                .lock()
                .unwrap()
                .remove_subscription(id)
            {
                true => Ok(Some(format!("Subscription with ID {} deleted", id))),
                // This is not considered an error.
                false => Ok(Some(
                    "Subscription already deleted or never existed".to_string(),
                )),
            }
        }

        // Current channel only.
        "/list" => {
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

        "/info" => {
            let id = message
                .content
                .strip_prefix(command)
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
        // set the name of a subscription. This will need an ID and the rest of the string will
        // be the name.
        // "/settings" => {}
        // "/seturl" => {}
        // "/setinterval" => {}
        //
        "/help" => Ok(Some(
            "
/help                   | Show this help message.
/add URL                | Add a new subscription. The URL is not checked for validity.
/del ID                 | Delete a subscription.
/list                   | List all subscriptions for the current channel.
/info ID                | Show metrics for a subscription.

variables:
ID: The subscription ID. You can get this by using /list.
URL: The URL to scrape. Only hardverapro is supported currently.
"
            .to_string(),
        )),

        _ => Ok(None),
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
}
