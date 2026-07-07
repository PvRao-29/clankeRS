//! ROS 2 integration for clankeRS — the ROS-free shared core.
//!
//! This crate is a member of the main Cargo workspace and builds with plain
//! `cargo build` (no ROS 2 install). It provides:
//!
//! * the in-memory **sim** backend (`RobotNode`/`Publisher`/`Subscriber`) used
//!   by CI, examples, and replay tests, and
//! * the transport-agnostic message + QoS types (`ImageMsg`, `DetectionArray`,
//!   `QosProfile`, `WireType`, ...) shared by *both* backends.
//!
//! The real rclrs/DDS backend does **not** live here — it can only be compiled
//! inside a colcon workspace (the ROS message crates are yanked on crates.io, so
//! declaring them anywhere the main workspace resolves would break the ROS-free
//! `cargo build`). It lives in a separate, checked-in colcon package,
//! [`ros2/clankers-ros2-dds`](https://github.com/PvRao-29/clankeRS/tree/main/ros2/clankers-ros2-dds),
//! which depends on *this* crate for the shared `message`/`qos` types and
//! re-exports the same `RobotNode`/`Publisher`/`Subscriber` API. See
//! `docs/ros2_integration.md`.

pub mod message;
pub mod node;
pub mod qos;
pub mod sim;

pub use message::{Detection, DetectionArray, ImageMsg, RosMessage, WireType};
pub use node::{inject_message, Publisher, RobotNode, Subscriber};
pub use qos::QosProfile;
