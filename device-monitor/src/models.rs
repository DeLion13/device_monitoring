use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

/// Map of device IDs to list of clients that subscribed on it
pub(crate) type Subscriptions = Arc<Mutex<HashMap<String, Vec<UnboundedSender<Message>>>>>;

/// Scheme to parse a subscription request
#[derive(Deserialize, Serialize)]
pub(crate) struct SubscribeRequest {
    pub device_id: String,
}
