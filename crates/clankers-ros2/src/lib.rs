//! ROS 2 integration for clankeRS.
//!
//! By default builds with the `sim` backend (in-memory pub/sub for replay and testing).
//! Enable the `ros2` feature for real ROS 2 integration when rclrs is available.

pub mod message;
pub mod node;
pub mod qos;
pub mod sim;

pub use message::{Detection, DetectionArray, ImageMsg, RosMessage};
pub use node::{inject_message, Publisher, RobotNode, Subscriber};
pub use qos::QosProfile;
