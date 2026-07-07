//! ROS 2 integration for clankeRS.
//!
//! Two backends, selected at compile time and sharing one public API
//! (`RobotNode`, `Publisher`, `Subscriber`):
//!
//! * **`sim`** (default): in-memory pub/sub for replay and testing. No ROS 2
//!   install required — this is what CI and `cargo build` use.
//! * **`ros2`**: real rclrs/DDS backend. Compiles only inside a colcon
//!   workspace (see [`rclrs_backend`] and `docs/ros2_integration.md`). When the
//!   `ros2` feature is on it takes precedence over `sim`.

pub mod message;
pub mod qos;

#[cfg(not(feature = "ros2"))]
pub mod node;
#[cfg(not(feature = "ros2"))]
pub mod sim;
#[cfg(not(feature = "ros2"))]
pub use node::{inject_message, Publisher, RobotNode, Subscriber};

#[cfg(feature = "ros2")]
pub mod bridge;
#[cfg(feature = "ros2")]
pub mod rclrs_backend;
#[cfg(feature = "ros2")]
pub use rclrs_backend::{Publisher, RobotNode, Subscriber};

pub use message::{Detection, DetectionArray, ImageMsg, RosMessage, WireType};
pub use qos::QosProfile;
