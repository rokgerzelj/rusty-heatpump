use super::{sensor_state::SensorState, thermostat_state::ThermostatState};

pub enum DeviceState {
    SensorState(SensorState),
    ThermostatState(ThermostatState),
}
