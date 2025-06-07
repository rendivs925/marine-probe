use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub timestamp: String,
    pub temperature: f64,
    pub salinity: f64,
    pub turbidity: f64,
}
