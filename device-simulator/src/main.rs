mod error;
mod models;
mod utils;

use error::DeviceSimulatorResult;
use models::*;
use utils::*;

/// Period of sending a temperature update (in seconds)
const TEMPERATURE_PERIOD_SEC: u64 = 3;

/// Period of sending a heartbeat update (in seconds)
const HEARTBEAT_PERIOD_SEC: u64 = 15;

#[tokio::main]
async fn main() -> DeviceSimulatorResult<()> {
    env_logger::init();

    // Initialize array of devices
    let devices = vec![1, 2, 3];
    let mut tasks = vec![];

    // Start up each device with given ID
    for device_id in devices {
        let task = tokio::spawn(async move { simulate_device(device_id).await });
        tasks.push(task);
    }

    // Run tasks for all devices simultaneously
    //
    // Each of tasks doesn't return anything but we care
    // about the possible configuration error that might
    // occur while sending the initialization message
    futures::future::try_join_all(tasks)
        .await?
        .into_iter()
        .collect::<DeviceSimulatorResult<()>>()
}

/// Creates device with given `ID` and starts sending
/// information to `POST /devices/{id}/messages` endpoint
async fn simulate_device(device_id: u32) -> DeviceSimulatorResult<()> {
    // Startup message without body
    let startup_message = DeviceLog {
        device_id: device_id.to_string(),
        message: "".to_string(),
    };
    send_message(device_id, &startup_message).await?;

    // Create temperature task
    let temperature_task = spawn_device_task(device_id, TEMPERATURE_PERIOD_SEC, || {
        serde_json::to_string(&TemperatureMessage {
            temperature: random_u64(50, 110) as u8,
        })
        .map_err(|err| err.into())
    });

    // Create heartbeat task
    let heartbeat_task = spawn_device_task(device_id, HEARTBEAT_PERIOD_SEC, || {
        serde_json::to_string(&HeartbeatMessage {
            cpu_usage: random_u64(0, 100) as u8,
            mem_usage: random_u64(1, 10000),
        })
        .map_err(|err| err.into())
    });

    // Run tasks simultaneously
    futures::future::try_join_all([temperature_task, heartbeat_task])
        .await?
        .into_iter()
        .collect::<DeviceSimulatorResult<()>>()
}
