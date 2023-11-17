use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Sensor {
    pub device_id: String,
}

#[derive(Clone)]
pub struct Thermostat {
    pub device_id: String,
}

#[derive(Clone)]
pub enum TRVTempControl {
    ExternalSensor, // configures TRV to only use external sensos, falls back to internal sensor if external sensor is not available
    Mixed,          // configures TRV to use both external sensor and internal sensor (auto offset)
    InternalSensor, // configures TRV to only use internal sensor
}

#[derive(Clone)]
pub struct Room {
    pub name: String,
    pub sensor: Sensor,
    pub thermostat: Thermostat,
    pub load_balancing: bool, // enables load balancing between TRVs in the room
    pub trv_temp_control: TRVTempControl, // This applies to all TRVs in the room (for now)
}

#[derive(Clone)]
pub struct Config {
    pub rooms: Vec<Room>,
}

impl Config {
    pub fn sensor_ids(&self) -> Vec<String> {
        self.rooms
            .iter()
            .map(|room| room.sensor.device_id.clone())
            .collect()
    }

    pub fn thermostat_ids(&self) -> Vec<String> {
        self.rooms
            .iter()
            .map(|room| room.thermostat.device_id.clone())
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorState {
    pub battery: u8,
    pub humidity: f64,
    pub linkquality: u8,
    pub temperature: f64,
    pub voltage: u16,
    last_seen: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThermostatState {
    adaptation_run_control: String,
    adaptation_run_settings: bool,
    adaptation_run_status: String,
    algorithm_scale_factor: i32,
    battery: i32,
    day_of_week: String,
    external_measured_room_sensor: i32,
    heat_available: bool,
    pub heat_required: bool,
    keypad_lockout: String,
    linkquality: i32,
    load_balancing_enable: bool,
    load_estimate: i32,
    load_room_mean: i32,
    local_temperature: f64,
    mounted_mode_active: bool,
    mounted_mode_control: bool,
    pub occupied_heating_setpoint: f64,
    occupied_heating_setpoint_scheduled: f64,
    pub pi_heating_demand: i32,
    preheat_status: bool,
    programming_operation_mode: String,
    radiator_covered: bool,
    regulation_setpoint_offset: f64,
    running_state: String,
    setpoint_change_source: String,
    system_mode: String,
    thermostat_vertical_orientation: bool,
    trigger_time: i32,
    window_open_external: bool,
    window_open_feature: bool,
    window_open_internal: String,
    last_seen: String,
}

impl ThermostatState {
    pub fn is_window_open(&self) -> bool {
        self.window_open_external
            || self.window_open_internal == "external_open"
            || self.window_open_internal == "open"
    }
}

pub struct StateTracker {
    thermostat_states: HashMap<String, ThermostatState>,
    sensor_states: HashMap<String, SensorState>,
    pub config: Config,
}

impl StateTracker {
    pub fn new(config: Config) -> Arc<RwLock<StateTracker>> {
        Arc::new(RwLock::new(StateTracker {
            thermostat_states: HashMap::new(),
            sensor_states: HashMap::new(),
            config,
        }))
    }

    pub fn update(&mut self, device_id: String, payload: String) {
        let sensor_ids = self.config.sensor_ids();
        let thermostat_ids = self.config.thermostat_ids();

        if sensor_ids.contains(&device_id) {
            let sensor_state: SensorState = serde_json::from_str(&payload).unwrap();
            self.sensor_states.insert(device_id, sensor_state);
        } else if thermostat_ids.contains(&device_id) {
            let thermostat_state: ThermostatState = serde_json::from_str(&payload).unwrap();
            self.thermostat_states
                .insert(device_id, thermostat_state.clone());
        } else {
            panic!("Unknown device_id: {}", device_id);
        }
    }

    // TODO: add a timestamp to the state so we can check if it's recent or stale
    // return None if the state is stale
    pub fn get_recent_sensor_state(&self, device_id: &str) -> Option<&SensorState> {
        self.sensor_states.get(device_id)
    }

    // TODO: add a timestamp to the state so we can check if it's recent or stale
    // return None if the state is stale
    pub fn get_recent_thermostat_state(&self, device_id: &str) -> Option<&ThermostatState> {
        self.thermostat_states.get(device_id)
    }

    // log all room states
    pub fn print_states(&self) {
        for room in &self.config.rooms {
            let sensor_state = self.sensor_states.get(&room.sensor.device_id);
            let thermostat_state = self.thermostat_states.get(&room.thermostat.device_id);

            println!(
                "{}: {}°C sen, {}% hum sen, {} ext temp, {} int temp, {}% heat demand, {} rad covered",
                room.name,
                sensor_state
                    .map(|s| format!("{:.1}", s.temperature))
                    .unwrap_or("UNK".to_string()),
                sensor_state
                    .map(|s| format!("{:.1}", s.humidity))
                    .unwrap_or("UNK".to_string()),
                thermostat_state
                    .map(|s| format!("{:.1}", s.external_measured_room_sensor))
                    .unwrap_or("UNK".to_string()),
                thermostat_state
                    .map(|s| format!("{:.1}", s.local_temperature))
                    .unwrap_or("UNK".to_string()),
                thermostat_state
                    .map(|s| format!("{}", s.pi_heating_demand))
                    .unwrap_or("UNK".to_string()),
                thermostat_state
                    .map(|s| format!("{}", s.radiator_covered))
                    .unwrap_or("UNK".to_string()),
            );
        }
    }
}
