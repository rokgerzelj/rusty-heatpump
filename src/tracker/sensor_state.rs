use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorState {
    pub battery: u8,
    pub humidity: f64,
    pub linkquality: u8,
    pub temperature: f64,
    pub voltage: u16,
}
