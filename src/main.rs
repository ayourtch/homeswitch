use async_std::task;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceDefinition {
    model: String,
    vendor: String,
    description: String,
    // option: ...
    // exposes: ...
    //
    //
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceEntry {
    ieee_address: String,
    #[serde(rename = "type")]
    typ: String,
    network_address: u32,
    supported: bool,
    // disabled: bool,
    friendly_name: String,
    // description: String,
    // endpoints: Vec<...>,
    // definition: DeviceDefinition,
    // power_source: String,
    // date_code: String,
    // model_id: String,
    // scenes:
}

#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "192.168.0.220", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(1000000, 1000000);

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("zigbee2mqtt/bridge/Xlogging", QoS::AtMostOnce)
        .await
        .unwrap();
    client
        .subscribe("zigbee2mqtt/bridge/devices", QoS::AtMostOnce)
        .await
        .unwrap();

    /*
     * let json_bytes: Vec<u8> = r#"{"brightness":56,"color":{"x":0.46187,"y":0.19485},"color_mode":"xy","color_temp":250,"state":"ON"}"#.into();
    client
        .publish(
            "zigbee2mqtt/Living Above Couch - 0x000b57fffea0074a/set",
            QoS::AtMostOnce,
            false,
            json_bytes,
        )
        .await
        .unwrap();

    */

    loop {
        let notification = eventloop.poll().await.unwrap();
        println!("Received = {:?}", notification);
        match notification {
            rumqttc::Event::Incoming(incoming) => match incoming {
                rumqttc::Packet::Publish(publish) => {
                    if publish.topic == "zigbee2mqtt/bridge/devices" {
                        let devices: Vec<DeviceEntry> =
                            serde_json::from_slice(&publish.payload).unwrap();

                        println!("Devices: {:?}", &devices);
                        for d in &devices {
                            if d.friendly_name.starts_with("Switch") {
                                client
                                    .subscribe(
                                        &format!("zigbee2mqtt/{}", &d.friendly_name),
                                        QoS::AtMostOnce,
                                    )
                                    .await
                                    .unwrap();
                            }
                        }
                    } else {
                        println!("publish: {:?}", &publish);
                        let s = String::from_utf8_lossy(&publish.payload);
                        println!("payload: {}", s);
                    }
                }
                x => {
                    println!("all other incoming: {:?}", &x);
                }
            },
            x => {
                println!("All other notification: {:?}", &x);
            }
        }
    }
}
