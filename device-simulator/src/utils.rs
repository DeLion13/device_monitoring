use crate::{error::*, DeviceLog};
use chrono::Utc;
use log::{debug, error, warn};
use rand::{rngs::StdRng, Rng, SeedableRng};
use reqwest::Client;
use std::time::Duration;
use tokio::{task::JoinHandle, time};

/// Service port to connect to
const PORT: u64 = 8080;

/// Localhost IP address
const IP_ADDRESS: &'static str = "127.0.0.1";

/// Random number generator of [`u64`]
///
/// This is analogue to how it's usually done in `C`
/// using current timestamp as a seed for randomizer
pub(crate) fn random_u64(min: u64, max: u64) -> u64 {
    let now = Utc::now().timestamp_micros() as u64;

    // Creating common `StdRng` for all tokio tasks wasn't a good approach
    // because it has to implement `Send + Sync` which the structure itself
    // doesn't. Wrapping it inside `Arc<Mutex<_>>` brings us to bulk cloning
    // of the reference. To use generator we need to first lock mutex, which
    // is not time efficient since we need it to be used simultaneously in few tasks
    let mut rng = StdRng::seed_from_u64(now as u64);

    rng.gen_range(min..=max)
}

/// Spawn device task with specific period and message generation
pub(crate) fn spawn_device_task<F>(
    device_id: u32,
    period_sec: u64,
    generate_message: F,
) -> JoinHandle<DeviceSimulatorResult<()>>
where
    F: Fn() -> DeviceSimulatorResult<String> + Send + 'static,
{
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(period_sec));

        loop {
            interval.tick().await;

            let message = generate_message()?;

            // Spawn separate task for the message sending
            // to avoid locking the interval tick
            tokio::spawn(async move {
                let device_message = DeviceLog {
                    device_id: device_id.to_string(),
                    message,
                };

                let _ = send_message(device_id, &device_message)
                    .await
                    .inspect_err(|err| error!("{err}"));
            });
        }

        // We won't ever reach this but we want to avoid panicking
        #[allow(unreachable_code)]
        Ok(())
    })
}

/// Send `POST` request with device ID and
/// message to the monitoring server
pub(crate) async fn send_message(device_id: u32, msg: &DeviceLog) -> DeviceSimulatorResult<()> {
    let client = Client::new();

    // Build URL for `POST` request
    let url = format!(
        "http://{}:{}/devices/{}/messages",
        IP_ADDRESS, PORT, device_id
    );

    // Error handling for the sent message
    match client.post(&url).json(&msg).send().await {
        Ok(response) => {
            if response.status().is_success() {
                debug!(
                    "Message sent successfully\n\nDevice ID: {}\nMessage: {:#?}",
                    device_id, msg
                );
            } else {
                // A bunch of possible failures that shouldn't
                // corrupt the whole system but should be logged
                warn!("Failed to deliver a message\n\nResponse: {:#?}", response)
            }
        }
        // Will occur if the request is misconfigured
        // or if the sending of the request has failed
        // for reasons like network loss or server shutdown
        Err(err) => {
            error!("Request error: {:#?}", err);

            return Err(DeviceSimulatorError::PostRequest { source: err });
        }
    }

    Ok(())
}
