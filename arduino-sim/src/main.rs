use chrono::Utc;
use rand::{Rng, rng};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;
use tokio::{task, time};

#[tokio::main]
async fn main() {
    let mut mqttoptions = MqttOptions::new("sensor_simulator", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    loop {
        if let Ok(event) = eventloop.poll().await {
            if let Event::Incoming(Packet::ConnAck(_)) = event {
                println!("✅ MQTT connected");
                break;
            }
        }
    }

    task::spawn(async move {
        while let Ok(event) = eventloop.poll().await {
            println!("🔄 Event: {:?}", event);
        }
    });

    let mut rng = rng();

    loop {
        let payload = format!(
            r#"{{"timestamp":"{}","temperature":{:.2},"salinity":{:.2},"turbidity":{:.2}}}"#,
            Utc::now().to_rfc3339(),
            rng.random_range(20.0..30.0),
            rng.random_range(30.0..35.0),
            rng.random_range(5.0..15.0)
        );

        if let Err(e) = client
            .publish("probe/data", QoS::AtLeastOnce, false, payload)
            .await
        {
            eprintln!("❌ Publish failed: {:?}", e);
        }

        time::sleep(Duration::from_secs(2)).await;
    }
}
