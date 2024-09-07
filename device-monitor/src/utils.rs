use crate::{SubscribeRequest, Subscriptions};
use futures::{stream::SplitSink, SinkExt};
use log::{error, info, warn};
use std::convert::Infallible;
use tokio::sync::mpsc::UnboundedSender;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

/// Add [`Subscriptions`] to the [`Filter`]
pub(crate) fn with_subscriptions(
    subscriptions: Subscriptions,
) -> impl Filter<Extract = (Subscriptions,), Error = Infallible> + Clone {
    warp::any().map(move || subscriptions.clone())
}

/// Spawn a special tokio task that redirects messages from device to `WebSocket` client
pub(crate) fn spawn_redirection_task(
    ws_tx: SplitSink<WebSocket, Message>,
) -> UnboundedSender<Message> {
    // Create a separate channel for a new subscription for the requested device
    let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut ws_tx = ws_tx;

    tokio::spawn(async move {
        // When channel receive a message, it redirects
        // it right to corresponding WebSocket client
        while let Some(message) = client_rx.recv().await {
            // Send message to WebSocket client
            if let Err(err) = ws_tx.send(message).await {
                error!("Failed to send message to WebSocket client: {err}");
                break;
            }
        }
    });

    client_tx
}

/// Parses message for [`SubscribeRequest`] and adds this subscription for client
///
/// In case of duplication
pub(crate) async fn subscribe_client_for_device(
    message: Message,
    subscriptions: Subscriptions,
    client_tx: UnboundedSender<Message>,
) {
    let text = match message.to_str() {
        Ok(text) => text,
        Err(_) => {
            warn!("Converting [`Message`] to [`&str`] failed");
            return;
        }
    };

    match serde_json::from_str::<SubscribeRequest>(text) {
        Ok(subscribe_request) => {
            let mut subs = subscriptions.lock().await;

            // Add new subscriber for `device_id`
            let vec = subs
                .entry(subscribe_request.device_id.clone())
                .or_insert_with(Vec::new);

            // Also possible to use HashSet, but for that I would
            // have to create a wrapper over [`UnboundedSender`]
            vec.push(client_tx.clone());
            vec.dedup_by(|a, b| a.same_channel(b));

            info!("User subscribed on device: {}", subscribe_request.device_id);
        }
        Err(err) => {
            warn!("Failed to parse [`SubscribeRequest`]: {err}");
        }
    };
}
