# Device Monitoring System

This application is split into two parts: the server (`device-monitor`) and the client (`device-simulator` crate). The client simulates a group of devices that send their metrics at varying intervals. The server receives these updates and forwards them to `WebSocket` clients that are subscribed to those devices.

Both parts must be running for the system to function correctly. However, both are designed to be fault-tolerant and independent, meaning that if one shuts down, it can be restarted without affecting the other.

## Components

### 1. `device-monitor` (Server)
The server provides a simple API consisting of two endpoints:

| Method | Endpoint                | Description                                                               |
|--------|-------------------------|---------------------------------------------------------------------------|
| `POST` | `/devices/{id}/messages`| Receives device-generated messages, where `{id}` is the device identifier.|
| `WS`   | `/`                     | Allows clients to connect via WebSocket to listen for updates.            |

The `POST` endpoint is used by the `device-simulator` to send messages. The WebSocket (`WS`) endpoint is for external clients that want to subscribe and listen for updates.

### 2. `device-simulator` (Client)
Simulates three devices, each sending the following data:

- **Every 15 seconds** (Heartbeat message):

    ```rust
    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct HeartbeatMessage {
        pub cpu_usage: u8,  // Usage in percentages (%)
        pub mem_usage: u64, // Memory usage in kilobytes (KB)
    }
    ```

- **Every 3 seconds** (Temperature message):

    ```rust
    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct TemperatureMessage {
        pub temperature: u8, // Temperature in Celsius (°C)
    }
    ```

- **On startup**: An empty message.

All messages are sent in JSON format to the server using the following unified message structure:

  ```rust
    #[derive(Serialize, Deserialize, Debug)]
    pub struct DeviceLog {
        pub device_id: String,
        pub message: String,
    }
  ```
## How to Use

1. **Clone the repository**:
    ```bash
    git clone git@github.com:DeLion13/device_monitoring.git
    ```

2. **Update the Rust toolchain**:
    ```bash
    rustup update
    ```

3. **Enter the project directory**:
    ```bash
    cd device_monitoring
    ```

4. **Start the server**:
    ```bash
    RUST_LOG=info cargo run --bin device-monitor
    ```

5. **Start the client**:
    ```bash
    RUST_LOG=debug cargo run --bin device-simulator
    ```

6. **Enable logging** (optional):  
   You can enable logging using the environment variable `RUST_LOG=<info/warn/debug/error>` to see logs for debugging or monitoring purposes. I suggested my logging levels in higher but they are not mandatory. You may use your own or avoid them at all.

7. **Testing with Postman**:  
   Use Postman to test the API and manage WebSocket connections.

8. **Connect to the server**:  
   Use the URL `ws://127.0.0.1:8080/` to connect via WebSocket.

9. **Subscribe to devices**:  
   To subscribe, send a message in the following format (only specified identifiers are available):

    ```json
    {
        "subscribe": "<1/2/3>"
    }
    ```

10. **View WebSocket client data**:  
    You will see the corresponding messages from the subscribed devices:
<img width="1417" alt="Screenshot 2024-09-07 at 18 20 18" src="https://github.com/user-attachments/assets/7a072794-b9f0-43cb-a0c1-99c90bf92821">


11. **Manage subscriptions**:  
    You may add as many subscriptions as you want for each connection.

12. **Handle disconnections**:  
    If you disconnect, the system will delete the information about that connection and won’t attempt to reconnect.

13. **Handle server downtime**:  
    If you shut down the `device-monitor`, the `device-simulator` will continue running but will log errors until the server is back up.

14. **Handle client downtime**:  
    If you shut down the `device-simulator`, the messages will stop, but as soon as you restart the simulation, messages will resume appearing on your WebSocket client.
