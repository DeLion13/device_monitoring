use serde::{Deserialize, Serialize};

/// Scheme to send message from devices
#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceLog {
    pub device_id: String,
    pub message: String,
}

/// Scheme of heartbeat message body
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HeartbeatMessage {
    /// Usage in `percents (%)`
    pub cpu_usage: u8,

    /// Memory usage in `kilobytes (KB)`
    pub mem_usage: u64,
}

/// Scheme of temperature message body
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TemperatureMessage {
    /// Temperature in `Celcius (C)`
    pub temperature: u8,
}
