//! # Robotics math, transforms, and frames
//!
//! Lightweight geometry types for poses, transforms, and twists. Intended for
//! future TF-style frame graphs; today these are standalone value types.
//!
//! ```no_run
//! use clankers_geometry::{Pose, Transform, Twist};
//!
//! let _pose = Pose::identity("base");
//! let tf = Transform::new("world", "base");
//! let (_x, _, _) = tf.transform_point(1.0, 0.0, 0.0);
//! ```

pub mod pose;
pub mod transform;
pub mod twist;

pub use pose::Pose;
pub use transform::Transform;
pub use twist::Twist;
