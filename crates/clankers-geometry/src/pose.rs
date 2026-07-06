use clankers_core::{FrameId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pose {
    pub frame_id: FrameId,
    pub stamp: Timestamp,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub qw: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
}

impl Pose {
    pub fn identity(frame: impl Into<String>) -> Self {
        Self {
            frame_id: FrameId::new(frame),
            stamp: Timestamp::now(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            qw: 1.0,
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
        }
    }
}
