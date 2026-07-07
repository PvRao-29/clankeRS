//! Real rclrs/DDS backend for clankeRS.
//!
//! Built by colcon inside a ROS 2 workspace (see `ros2/README.md`). It depends
//! on the [`clankers_ros2`] crate for the shared, transport-agnostic message and
//! QoS types, and re-exports `RobotNode` / `Publisher` / `Subscriber` with the
//! same API as the sim backend, so node code is identical across transports.
//!
//! ```ignore
//! use clankers_ros2_dds::{RobotNode, ImageMsg, QosProfile};
//!
//! let node = RobotNode::new("my_node").await?;
//! let publisher = node.publish::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data()).await?;
//! ```

pub mod bridge;
pub mod rclrs_backend;

pub use rclrs_backend::{Publisher, RobotNode, Subscriber};

// Re-export the shared types so DDS example/node code has the same surface as
// `clankers::prelude` (which pulls these from `clankers_ros2`).
pub use clankers_core::{RobotError, RobotResult};
pub use clankers_ros2::{Detection, DetectionArray, ImageMsg, QosProfile, RosMessage, WireType};
