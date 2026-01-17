pub fn add(message: IncomingMessage, context: AppCtx) -> Result<Option<String>, String> {
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
