//! Conversions between clankeRS message types and real ROS 2 message types.
//!
//! Compiled only under `--features ros2` (needs the colcon-generated message
//! crates). Pure struct mapping — no DDS — so the conversions are unit-testable
//! in a ROS build without a running graph.

pub mod image;
