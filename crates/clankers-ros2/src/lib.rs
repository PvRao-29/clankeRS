//! # ROS 2 integration (sim backend + shared types)
//!
//! This crate builds with plain `cargo build` — **no ROS 2 install required**.
//!
//! It provides:
//!
//! * the in-memory **sim** pub/sub backend ([`RobotNode`], [`Publisher`], [`Subscriber`])
//!   used by CI, examples, and replay tests, and
//! * transport-agnostic message and QoS types ([`ImageMsg`], [`DetectionArray`],
//!   [`QosProfile`]) shared by both sim and real DDS backends.
//!
//! ## Quick start — subscribe and publish
//!
//! ```no_run
//! use clankers_ros2::{DetectionArray, ImageMsg, QosProfile, RobotNode};
//!
//! #[tokio::main]
//! async fn main() -> clankers_core::RobotResult<()> {
//!     let node = RobotNode::new("perception").await?;
//!     let mut images = node
//!         .subscribe::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data())
//!         .await?;
//!     let _detections = node
//!         .publish::<DetectionArray>("/detections", QosProfile::default())
//!         .await?;
//!
//!     while let Some(_frame) = images.next().await {
//!         // publish detections when your pipeline produces them
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Real DDS / rclrs
//!
//! The production DDS backend is **not** in this crate (ROS message crates are yanked
//! on crates.io). It lives in the checked-in colcon package
//! [`ros2/clankers-ros2-dds`](https://github.com/PvRao-29/clankeRS/tree/main/ros2/clankers-ros2-dds)
//! and re-exports the same API. See `docs/ros2_integration.md` in the repo.

pub mod message;
pub mod node;
pub mod qos;
pub mod sim;

pub use message::{Detection, DetectionArray, ImageMsg, RosMessage, WireType};
pub use node::{inject_message, Publisher, RobotNode, Subscriber};
pub use qos::QosProfile;
