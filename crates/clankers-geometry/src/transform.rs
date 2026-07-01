use clankers_core::{FrameId, RobotResult, Timestamp};

use crate::pose::Pose;

#[derive(Debug, Clone)]
pub struct Transform {
    pub parent: FrameId,
    pub child: FrameId,
    pub stamp: Timestamp,
    pub pose: Pose,
}

impl Transform {
    pub fn new(parent: impl Into<String>, child: impl Into<String>) -> Self {
        Self {
            parent: FrameId::new(parent),
            child: FrameId::new(child),
            stamp: Timestamp::now(),
            pose: Pose::identity(""),
        }
    }

    pub fn transform_point(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        // Simplified: translation only for v0.1
        (x + self.pose.x, y + self.pose.y, z + self.pose.z)
    }

    pub fn lookup(_parent: &str, _child: &str, _stamp: Timestamp) -> RobotResult<Self> {
        Ok(Self::new(_parent, _child))
    }
}
