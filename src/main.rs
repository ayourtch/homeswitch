use async_std::task;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    actions: HashMap<String, HashMap<String, Vec<(String, String)>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceEvent {
    action: Option<String>,
    linkquality: Option<u8>,
    battery: Option<u8>,
}

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
    let mut cfg: Config = Default::default();

    let mut act: Vec<(String, String)> = Default::default();

    act.push(("asd".to_string(), r#"{ "state": "TOGGLE" }"#.to_string()));
    act.push(("dedas".to_string(), r#"{"state": "ON"}"#.to_string()));

    let mut map1: HashMap<String, Vec<(String, String)>> = Default::default();

    map1.insert("action_push".to_string(), act);
    cfg.actions.insert("switch1 - test".to_string(), map1);
    println!("Config: {}", toml::to_string(&cfg).unwrap());

    let cfg = std::fs::read_to_string("config.toml").unwrap();

    let config: Config = toml::from_str(&cfg).unwrap();

    println!("Config: {:?}", &config);

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
    client
        .subscribe("zigbee2mqtt/+", QoS::AtMostOnce)
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

                        println!("Devices:");

                        for d in &devices {
                            println!("{}", &d.friendly_name);
                            /*
                            if !d.friendly_name.starts_with("Living") {
                                println!("Subscribe {}", &d.friendly_name);
                                client
                                    .subscribe(
                                        &format!("zigbee2mqtt/{}", &d.friendly_name),
                                        QoS::AtMostOnce,
                                    )
                                    .await
                                    .unwrap();
                            }
                            */
                        }
                        println!("------");
                    } else {
                        let key = publish.topic.clone().replace("zigbee2mqtt/", "");
                        if let Some(cfg) = config.actions.get(&key) {
                            println!("Found key {}", &key);
                            let s = String::from_utf8_lossy(&publish.payload);
                            println!("payload: {}", s);
                            let event: DeviceEvent =
                                serde_json::from_slice(&publish.payload).unwrap();
                            println!("Event: {:?}", &event);
                            if let Some(action) = event.action {
                                if let Some(actions) = cfg.get(&action) {
                                    println!(
                                        "Found list of actions for event {}: {:?}",
                                        &action, &actions
                                    );
                                    for (dev, payload) in actions {
                                        let client = client.clone();
                                        let payload = payload.clone().replace("'", r#"""#);
                                        let target = format!("zigbee2mqtt/{}/set", dev);
                                        task::spawn(async move {
                                            client
                                                .publish(
                                                    &target,
                                                    QoS::AtMostOnce,
                                                    false,
                                                    payload.as_bytes(),
                                                )
                                                .await
                                                .unwrap();
                                        });
                                    }
                                }
                            }
                        } else {
                            println!("publish: {:?}", &publish);
                            let s = String::from_utf8_lossy(&publish.payload);
                            println!("payload: {}", s);
                        }
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
