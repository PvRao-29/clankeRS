use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Twist {
    pub linear_x: f64,
    pub linear_y: f64,
    pub linear_z: f64,
    pub angular_x: f64,
    pub angular_y: f64,
    pub angular_z: f64,
}

impl Twist {
    pub fn zero() -> Self {
        Self {
            linear_x: 0.0,
            linear_y: 0.0,
            linear_z: 0.0,
            angular_x: 0.0,
            angular_y: 0.0,
            angular_z: 0.0,
        }
    }

    pub fn bounded_velocity(max_linear: f64, max_angular: f64) -> Self {
        Self {
            linear_x: max_linear,
            linear_y: max_linear,
            linear_z: max_linear,
            angular_x: max_angular,
            angular_y: max_angular,
            angular_z: max_angular,
        }
    }
}
