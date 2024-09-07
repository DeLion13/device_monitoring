mod models;
mod utils;

use device_simulator::DeviceLog;
use futures::{lock::Mutex, StreamExt};
use log::{debug, error, info, warn};
use models::*;
use std::{collections::HashMap, sync::Arc};
use utils::*;
use warp::{
    ws::{Message, WebSocket, Ws},
    Filter,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    // Create a global instance that keeps track of all subscriptions
    let subscriptions: Subscriptions = Arc::new(Mutex::new(HashMap::new()));

    // Process POST requests for this specific path
    let post_route = warp::path!("devices" / u32 / "messages")
        .and(with_subscriptions(subscriptions.clone()))
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_post);

    // Process `WS /` requests
    let ws_route = warp::path::end()
        .and(warp::ws())
        .and(with_subscriptions(subscriptions.clone()))
        .map(|ws: Ws, subs: Subscriptions| ws.on_upgrade(move |socket| handle_ws(socket, subs)));

    // Run the server on localhost with opened 8080 port
    warp::serve(post_route.or(ws_route))
        .run(([127, 0, 0, 1], 8080))
        .await;
}

/// Handles POST request and immediately redirects messages to the subscribers
async fn handle_post(
    id: u32,
    subscriptions: Subscriptions,
    message: DeviceLog,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("Received POST-request from device `{}`: {:#?}", id, message);

    let subs = subscriptions.lock().await;
    if let Some(subscribers) = subs.get(&message.device_id) {
        for subscriber in subscribers {
            let msg = match serde_json::to_string(&message) {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("Serialization has failed: {err}");
                    continue;
                }
            };
            if let Err(err) = subscriber.send(Message::text(msg)) {
                error!("Failed sending via WebSocket: {err}");
            }
        }
    }

    Ok(warp::reply::json(&"Messages sent"))
}

/// Handles each WS connection and creates a separate client for subscriptions
async fn handle_ws(ws: WebSocket, subscriptions: Subscriptions) {
    let (ws_tx, mut ws_rx) = ws.split();

    // This is where the magic of redirection happens
    let client_tx = spawn_redirection_task(ws_tx);

    while let Some(result) = ws_rx.next().await {
        match result {
            // We need to handle each message using pattern matching but unfortunately
            // the `inner` field of `Message` type from `warp` crate that would allow this
            // is private and we only have access to methods with boolean results.
            //
            // In this block we cover all message types we're interested at
            Ok(message) => {
                if message.is_text() {
                    subscribe_client_for_device(message, subscriptions.clone(), client_tx.clone())
                        .await;
                } else if message.is_close() {
                    info!("WebSocket connection is closed\nMessage: {message:#?}");

                    // Remove client-specific sender from subscribers.
                    // That action will destroy the task with receiver
                    unsubscribe_client(subscriptions.clone(), client_tx.clone()).await;
                }
            }
            Err(err) => {
                error!("Error occured for WebSocket: {err}");
                return;
            }
        }
    }
}
