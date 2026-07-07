//! clankeRS — Rust SDK for robotics applications in the ROS 2 ecosystem.

pub mod prelude;

pub use clankers_core::{
    ClankeRSConfig, LatencyStats, RobotContext, RobotError, RobotResult, Timestamp, TopicName,
};
pub use clankers_data::{InspectReport, McapLog, Replay, ReplayResult};
pub use clankers_geometry::{Pose, Transform, Twist};
pub use clankers_macros::{node, replay_test};
pub use clankers_ml::{Model, ModelBuilder, ModelValidator, ValidationReport};
pub use clankers_ros2::{
    Detection, DetectionArray, ImageMsg, Publisher, QosProfile, RobotNode, Subscriber,
};
// `inject_message` feeds the in-memory sim bus (used by replay). The `clankers`
// crate always uses the sim backend; the real rclrs/DDS backend is a separate
// colcon package (ros2/clankers-ros2-dds) where messages arrive over DDS.
pub use clankers_ros2::inject_message;
pub use clankers_runtime::{RobotRuntime, RuntimeMetrics};
pub use clankers_tensor::ImageTensor;
pub use clankers_testing::{
    assert_dropped_messages, assert_max_latency, assert_no_panics, assert_topic_exists,
    ReplayContext, ReplayTestResult,
};

pub mod runtime {
    pub use clankers_runtime::runtime::*;
}

pub mod testing {
    pub use clankers_testing::*;
}

pub mod data {
    pub use clankers_data::*;
}

pub mod ml {
    pub use clankers_ml::*;
}

pub mod tensor {
    pub use clankers_tensor::*;
}

pub mod ros2 {
    pub use clankers_ros2::*;
}
