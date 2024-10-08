mod state;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde_json::json;
use state::{Config, Room, Sensor, StateTracker, TRVTempControl, Thermostat};

async fn subscribe(client: &AsyncClient, config: &Config) {
    for id in config.sensor_ids() {
        client
            .subscribe(&format!("zigbee2mqtt/{}", id), QoS::AtLeastOnce)
            .await
            .unwrap();
    }

    for id in config.thermostat_ids() {
        client
            .subscribe(&format!("zigbee2mqtt/{}", id), QoS::AtLeastOnce)
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    let config = Config {
        rooms: vec![Room {
            name: "Kopalnica".to_string(),
            sensor: Sensor {
                device_id: "0xa4c1385a6271b083".to_string(),
            },
            thermostat: Thermostat {
                device_id: "0x3410f4fffe617bcc".to_string(),
            },
            load_balancing: false,
            trv_temp_control: TRVTempControl::ExternalSensor,
        }],
    };

    let mqtt_options = MqttOptions::new("rust_client", "192.168.0.40", 1883)
        .set_max_packet_size(5 * 1024 * 1024, 5 * 1024 * 1024)
        .to_owned();

    let (client, mut event_loop) = AsyncClient::new(mqtt_options, 50);

    subscribe(&client, &config).await;

    let state_tracker = StateTracker::new(config.clone());

    // TODO: try to "poke" the thermostats (and maybe sensors??) to get the state on startup

    tokio::spawn({
        let client = client.clone();
        let state_tracker = state_tracker.clone();

        async move {
            loop {
                // TODO: ensure states are recent and not stale, needs a state received_at timestamp.

                // !ORDER IMPORTANT! so we don't lock the state_tracker for the sleep duration
                tokio::time::sleep(tokio::time::Duration::from_secs(60 * 5)).await;
                let state_reader = state_tracker.read().await;

                for room in state_reader.config.rooms.iter() {
                    let reader = state_tracker.read().await;
                    let sensor_state = reader.get_recent_sensor_state(&room.sensor.device_id);

                    if let Some(sensor_state) = sensor_state {
                        let setting = json!({
                            "external_measured_room_sensor": (sensor_state.temperature * 100.0) as i32
                        });

                        client
                            .publish(
                                "zigbee2mqtt/".to_owned()
                                    + &room.thermostat.device_id.clone()
                                    + "/set",
                                QoS::AtLeastOnce,
                                false,
                                setting.to_string(),
                            )
                            .await
                            .unwrap();

                        println!(
                            "Setting external_measured_room_sensor for {} to {}",
                            room.name, sensor_state.temperature
                        );
                    } else {
                        println!(
                            "Missing recent sensor state for room {}, external_measured_room_sensor will not be set",
                            room.name
                        );
                    }
                }
            }
        }
    });

    tokio::spawn({
        let client = client.clone();
        let state_tracker = state_tracker.clone();

        async move {
            loop {
                {
                    let state_reader = state_tracker.read().await;
                    state_reader.print_states();
                }

                // !ORDER IMPORTANT! so we don't lock the state_tracker for the sleep duration
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                let state_reader = state_tracker.read().await;

                // loop over rooms and check if we need to turn on/off the heating
                // TODO: make this work for multiple rooms, currently each room will override the previous one's adjustment
                for room in state_reader.config.rooms.iter() {
                    let sensor_state = state_reader.get_recent_sensor_state(&room.sensor.device_id);
                    let thermostat_state =
                        state_reader.get_recent_thermostat_state(&room.thermostat.device_id);

                    if let (Some(sensor_state), Some(thermostat_state)) =
                        (sensor_state, thermostat_state)
                    {
                        let heat_required = thermostat_state.heat_required;
                        let heating_demand = thermostat_state.pi_heating_demand as f64 / 100.0; // scale to [0.0, 1.0]
                        let sensor_temp = sensor_state.temperature;
                        let setpoint = thermostat_state.occupied_heating_setpoint;

                        let mut adjustment = 0.0;
                        let demand_threshold = 0.75;
                        if heating_demand > demand_threshold {
                            // positive if we need to heat up
                            adjustment = (setpoint - sensor_temp)
                                * ((heating_demand - demand_threshold) / (1.0 - demand_threshold));
                            if adjustment < 0.0 {
                                adjustment = 0.0;
                            }
                            if adjustment > 2.0 {
                                adjustment = 2.0;
                            }
                        }

                        if !heat_required {
                            // this will turn off the heat pump completely
                            adjustment = -50.0;
                        }

                        let fixed_hp_temp = 21.5 - 0.1; // TODO: get this from HP state
                        let setting = format!("I10000={:.1}", fixed_hp_temp - adjustment);

                        // publish to bsblan heating circuit 1
                        client
                            .publish("BSB-LAN", QoS::AtLeastOnce, false, setting)
                            .await
                            .unwrap();

                        println!(
                            "Setting flow temp adjustment for {} to {:.1}",
                            room.name, adjustment
                        );
                    } else {
                        println!(
                            "Missing recent sensor or thermostat state for room {}, flow temp adjustment will not be set",
                            room.name
                        );
                    }
                }
            }
        }
    });

    loop {
        match event_loop.poll().await {
            Ok(event) => match event {
                rumqttc::Event::Incoming(rumqttc::Incoming::Publish(p)) => {
                    let topic = p.topic.to_string();
                    let payload_str = String::from_utf8(p.payload.to_vec()).unwrap();

                    println!("Topic: {}, Payload: {}", topic, payload_str);

                    let device_id: Vec<&str> = topic.split('/').collect();
                    if device_id.len() == 2 {
                        let id = device_id[1].to_string();

                        let mut state_tracker = state_tracker.write().await;
                        state_tracker.update(id, payload_str.clone());
                    }
                }
                rumqttc::Event::Incoming(rumqttc::Incoming::ConnAck(_ack)) => {
                    // On reconnect we have to resubscribe to the topics, rumqtt does not do it by
                    println!("CONNACK received, resubscribing to topics");
                    subscribe(&client, &config).await;
                }
                _ => {}
            },
            Err(e) => {
                eprintln!("Error in MQTT event loop: {:?}", e);
                // TODO: Handle error
            }
        }
    }
}
