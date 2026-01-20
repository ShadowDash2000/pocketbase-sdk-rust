use anyhow::anyhow;
use pocketbase_sdk::client::Client;
use pocketbase_sdk::realtime::EventResponse;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_realtime_connect_integration() {
    let client = Client::new("http://127.0.0.1:8090");

    let (data_tx, mut data_rx) = tokio::sync::mpsc::channel::<EventResponse>(1);

    client
        .collection("users")
        .subscribe("*", move |data| {
            let _ = data_tx.try_send(data);
        })
        .await
        .expect("Subscription failed");

    let received_data = tokio::time::timeout(Duration::from_secs(60), data_rx.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Channel closed");

    received_data
        .record
        .get("id")
        .ok_or_else(|| anyhow!("No id in event data: {:?}", data_rx.try_recv().unwrap()))
        .expect("TODO: panic message");
}
