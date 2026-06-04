/// ROS 2 QoS profile helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QosProfile {
    pub reliability: Reliability,
    pub durability: Durability,
    pub history_depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reliability {
    Reliable,
    BestEffort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    Volatile,
    TransientLocal,
}

impl QosProfile {
    pub fn default_profile() -> Self {
        Self {
            reliability: Reliability::Reliable,
            durability: Durability::Volatile,
            history_depth: 10,
        }
    }

    pub fn sensor_data() -> Self {
        Self {
            reliability: Reliability::BestEffort,
            durability: Durability::Volatile,
            history_depth: 5,
        }
    }
}

impl Default for QosProfile {
    fn default() -> Self {
        Self::default_profile()
    }
}
