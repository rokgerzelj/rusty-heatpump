use serde::{Deserialize, Serialize};

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
}

impl ThermostatState {
    pub fn is_window_open(&self) -> bool {
        self.window_open_external
            || self.window_open_internal == "external_open"
            || self.window_open_internal == "open"
    }
}
