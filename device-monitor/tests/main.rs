use device_simulator::DeviceLog;
use futures::{SinkExt, StreamExt};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn integration_testing() {
    // Start up the application on the background
    tokio::spawn(async {
        let routes = device_monitor::create_routes().await;
        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    });

    // Ensure it's up
    tokio::time::sleep(Duration::from_secs(3)).await;

    let client = Client::new();

    // Test the simplest case of POST request
    let response = client
        .post("http://127.0.0.1:3030/devices/1/messages")
        .json(&json!({
            "device_id": "1",
            "message": "foo"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success(), "POST request failed");

    // Test the WS scenario
    let (mut ws_stream, _) = connect_async("ws://127.0.0.1:3030/")
        .await
        .expect("Failed to connect");

    // Subscribe for device `1`
    ws_stream
        .send(Message::text(
            r#"{
            "subscribe": "1"
        }"#,
        ))
        .await
        .unwrap();

    // Send a simulated message from device `1`
    client
        .post("http://127.0.0.1:3030/devices/1/messages")
        .json(&json!({
            "device_id": "1",
            "message": "foo_baz_bar"
        }))
        .send()
        .await
        .expect("Failed to send POST request");

    // Wait for the message to appear on the client
    //
    // Hangs if there no messages coming to the client
    // so we need to set a timeout for that case
    if let Ok(Some(Ok(Message::Text(msg)))) =
        tokio::time::timeout(Duration::from_secs(3), ws_stream.next()).await
    {
        assert_eq!(
            // Parse and turn into [`Value`] for easy comparison without deriving
            serde_json::to_value(serde_json::from_str::<DeviceLog>(&msg).unwrap()).unwrap(),
            json!({
                "device_id": "1",
                "message": "foo_baz_bar"
            })
        );
    } else {
        panic!("No message received via WebSocket");
    }
}
