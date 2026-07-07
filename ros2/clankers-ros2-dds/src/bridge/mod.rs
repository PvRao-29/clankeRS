//! Conversions between clankeRS message types and real ROS 2 message types.
//!
//! Pure struct mapping — no DDS — so the conversions are unit-testable in a ROS
//! build without a running graph. The ROS types come from the colcon-generated
//! message crates (`sensor_msgs`, `std_msgs`, `builtin_interfaces`), declared in
//! `package.xml`.

pub mod image;
