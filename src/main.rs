mod config;
mod tracker {
    pub mod device_state;
    pub mod sensor_state;
    pub mod thermostat_state;
}
mod util;
mod mqtt {
    pub mod client;
}

use config::Config;
use mqtt::client::{MqttClient, MqttMessage};
use serde_json::Error;
use sqlx::sqlite::SqlitePool;
use tracing::{error, info};
use tracing_subscriber;
use tracker::{
    device_state::DeviceState,
    sensor_state::SensorState,
    thermostat_state::{self, ThermostatState},
};

enum ParseMessageError {
    JsonError(serde_json::Error),
    UnrecognizableDeviceId,
    UnrecognizableTopicName,
}

// fn parse_message(msg: MqttMessage, config: &Config) -> Result<DeviceState, ParseMessageError> {
//     let device_id: Vec<&str> = msg.topic.split('/').collect();
//     if device_id.len() == 2 {
//         let id = device_id[1].to_string();

//         if config.sensor_ids().contains(&id) {
//             let sensor_state: SensorState = serde_json::from(&msg.payload)?;
//             Ok(DeviceState::SensorState(sensor_state))
//         } else if config.thermostat_ids().contains(&id) {
//             let thermostat_state: ThermostatState = serde_json::from_str(&msg.payload)?;
//             Ok(DeviceState::ThermostatState(thermostat_state))
//         } else {
//             info!("received unrecognizable device id");
//             Err(ParseMessageError::UnrecognizableDeviceId)
//         }
//     } else {
//         info!("received unrecognizable topic name");
//         Err(ParseMessageError::UnrecognizableTopicName)
//     }
// }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config_str = util::read_file_to_string("config.yaml").await.unwrap();
    let config = Config::parse(config_str).unwrap();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let mqtt_app = MqttClient::new(config.mqtt_host.clone(), config.mqtt_port.clone());

    let (_, mut rx, mqtt_client) = mqtt_app.run();

    for id in config.sensor_ids() {
        mqtt_client
            .subscribe(&format!("zigbee2mqtt/{}", id), rumqttc::QoS::AtLeastOnce)
            .await
            .unwrap();
    }

    for id in config.thermostat_ids() {
        mqtt_client
            .subscribe(&format!("zigbee2mqtt/{}", id), rumqttc::QoS::AtLeastOnce)
            .await
            .unwrap();
    }

    loop {
        let msg = rx.recv().await.unwrap();

        let asd = async move {
            // check if the message is a sensor message or a thermostat message
            let topic = msg.topic.clone();
            let payload_str = String::from_utf8(msg.payload.to_vec());

            let device_id: Vec<&str> = topic.split('/').collect();
        };
    }

    // loop {
    //     let event = event_loop.poll().await.unwrap_or_else(|e| {
    //         error!("rumqqtc polling error: {:?}", e);
    //     });

    //     info!("processed event: {:?}", event);

    //     match event {
    //         rumqttc::Event::Incoming(rumqttc::Incoming::Publish(p)) => {
    //             let topic = p.topic.to_string();
    //             let payload_str = String::from_utf8(p.payload.to_vec()).unwrap();

    //             println!("Topic: {}, Payload: {}", topic, payload_str);

    //             let device_id: Vec<&str> = topic.split('/').collect();
    //             if device_id.len() == 2 {
    //                 let id = device_id[1].to_string();

    //                 let mut state_tracker = state::StateTracker::new(pool.clone());
    //                 state_tracker.update(id, payload_str.clone()).await;
    //             }
    //         }
    //         _ => {}
    //     }
    // }

    // loop {
    //     match event_loop.poll().await {
    //         Ok(event) => match event {
    //             rumqttc::Event::Incoming(rumqttc::Incoming::Publish(p)) => {
    //                 let topic = p.topic.to_string();
    //                 let payload_str = String::from_utf8(p.payload.to_vec()).unwrap();

    //                 println!("Topic: {}, Payload: {}", topic, payload_str);

    //                 let device_id: Vec<&str> = topic.split('/').collect();
    //                 if device_id.len() == 2 {
    //                     let id = device_id[1].to_string();

    //                     let mut state_tracker = state_tracker.write().await;
    //                     state_tracker.update(id, payload_str.clone());
    //                 }
    //             }
    //             _ => {}
    //         },
    //         Err(e) => {
    //             eprintln!("Error in MQTT event loop: {:?}", e);
    //             // Handle error as appropriate for your application
    //         }
    //     }
    // }
}
