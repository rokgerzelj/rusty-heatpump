use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Sensor {
    pub device_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Thermostat {
    pub device_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum TRVTempControl {
    ExternalSensor, // configures TRV to only use external sensos, falls back to internal sensor if external sensor is not available
    Mixed,          // configures TRV to use both external sensor and internal sensor (auto offset)
    InternalSensor, // configures TRV to only use internal sensor
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Room {
    pub name: String,
    pub sensor: Sensor,
    pub thermostats: Vec<Thermostat>,
    pub load_balancing: bool, // enables load balancing between TRVs in the room
    pub trv_temp_control: TRVTempControl, // This applies to all TRVs in the room (for now)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub rooms: Vec<Room>,
}

impl Config {
    pub fn parse(config_str: String) -> Result<Config, serde_yaml::Error> {
        serde_yaml::from_str(&config_str)
    }

    pub fn sensor_ids(&self) -> Vec<String> {
        self.rooms
            .iter()
            .map(|room| room.sensor.device_id.clone())
            .collect()
    }

    pub fn thermostat_ids(&self) -> Vec<String> {
        self.rooms
            .iter()
            .flat_map(|room| room.thermostats.iter().map(|t| t.device_id.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let config_str = r#"
mqtt_host: "localhost"
mqtt_port: 1883
rooms:
  - name: "Bathroom"
    sensor:
      device_id: "0xa4c1385a6271b083"
    thermostats:
      - device_id: "0x3410f4fffe617bcc"
    load_balancing: false
    trv_temp_control: "ExternalSensor"
        "#;
        let config = Config::parse(config_str.to_string()).unwrap();

        assert_eq!(config.rooms.len(), 1);
        assert_eq!(config.rooms[0].name, "Bathroom");
        assert_eq!(config.rooms[0].sensor.device_id, "0xa4c1385a6271b083");
        assert_eq!(
            config.rooms[0].thermostats[0].device_id,
            "0x3410f4fffe617bcc"
        );
        assert_eq!(config.rooms[0].load_balancing, false);
        assert_eq!(
            config.rooms[0].trv_temp_control,
            TRVTempControl::ExternalSensor
        );
    }

    #[test]
    fn sensor_ids() {
        let config = Config {
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1883,
            rooms: vec![
                Room {
                    name: "Bathroom".to_string(),
                    sensor: Sensor {
                        device_id: "0xa4c1385a6271b083".to_string(),
                    },
                    thermostats: vec![Thermostat {
                        device_id: "0x3410f4fffe617bcc".to_string(),
                    }],
                    load_balancing: false,
                    trv_temp_control: TRVTempControl::ExternalSensor,
                },
                Room {
                    name: "Bedroom".to_string(),
                    sensor: Sensor {
                        device_id: "0xa4c1385a6271b084".to_string(),
                    },
                    thermostats: vec![Thermostat {
                        device_id: "0x3410f4fffe617bcd".to_string(),
                    }],
                    load_balancing: false,
                    trv_temp_control: TRVTempControl::ExternalSensor,
                },
            ],
        };

        assert_eq!(
            config.sensor_ids(),
            vec!["0xa4c1385a6271b083", "0xa4c1385a6271b084"]
        );
    }

    #[test]
    fn thermostat_ids() {
        let config = Config {
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1883,
            rooms: vec![
                Room {
                    name: "Bathroom".to_string(),
                    sensor: Sensor {
                        device_id: "0xa4c1385a6271b083".to_string(),
                    },
                    thermostats: vec![
                        Thermostat {
                            device_id: "test".to_string(),
                        },
                        Thermostat {
                            device_id: "abc".to_string(),
                        },
                    ],
                    load_balancing: false,
                    trv_temp_control: TRVTempControl::ExternalSensor,
                },
                Room {
                    name: "Bedroom".to_string(),
                    sensor: Sensor {
                        device_id: "0xa4c1385a6271b084".to_string(),
                    },
                    thermostats: vec![Thermostat {
                        device_id: "hmm".to_string(),
                    }],
                    load_balancing: false,
                    trv_temp_control: TRVTempControl::ExternalSensor,
                },
            ],
        };

        assert_eq!(config.thermostat_ids(), vec!["test", "abc", "hmm"]);
    }
}
