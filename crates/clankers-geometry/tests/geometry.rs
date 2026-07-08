//! Integration tests for poses, transforms, and twists.

use clankers_core::{FrameId, Timestamp};
use clankers_geometry::{Pose, Transform, Twist};

#[test]
fn pose_identity() {
    let pose = Pose::identity("base_link");
    assert_eq!(pose.frame_id.as_str(), "base_link");
    assert_eq!((pose.x, pose.y, pose.z), (0.0, 0.0, 0.0));
    assert_eq!(pose.qw, 1.0);
    assert_eq!((pose.qx, pose.qy, pose.qz), (0.0, 0.0, 0.0));
}

#[test]
fn transform_point_applies_translation() {
    let mut tf = Transform::new("map", "base_link");
    tf.pose.x = 1.0;
    tf.pose.y = 2.0;
    tf.pose.z = 3.0;
    assert_eq!(tf.transform_point(10.0, 20.0, 30.0), (11.0, 22.0, 33.0));

    assert_eq!(tf.parent.as_str(), "map");
    assert_eq!(tf.child.as_str(), "base_link");
}

#[test]
fn transform_lookup_stub_returns_frames() {
    let tf = Transform::lookup("odom", "base_link", Timestamp::from_nanos(0)).unwrap();
    assert_eq!(tf.parent.as_str(), "odom");
    assert_eq!(tf.child.as_str(), "base_link");
}

#[test]
fn twist_zero_and_bounded() {
    let zero = Twist::zero();
    assert_eq!(zero.linear_x, 0.0);
    assert_eq!(zero.angular_z, 0.0);

    let bounded = Twist::bounded_velocity(1.5, 0.8);
    assert_eq!(bounded.linear_x, 1.5);
    assert_eq!(bounded.angular_z, 0.8);
}

#[test]
fn pose_serde_round_trip() {
    let pose = Pose {
        frame_id: FrameId::new("camera"),
        stamp: Timestamp::from_nanos(1_234),
        x: 1.0,
        y: -2.0,
        z: 0.5,
        qw: 1.0,
        qx: 0.0,
        qy: 0.0,
        qz: 0.0,
    };
    let json = serde_json::to_string(&pose).unwrap();
    let back: Pose = serde_json::from_str(&json).unwrap();
    assert_eq!(pose, back);
}

#[test]
fn twist_serde_round_trip() {
    let twist = Twist::bounded_velocity(2.0, 1.0);
    let json = serde_json::to_string(&twist).unwrap();
    let back: Twist = serde_json::from_str(&json).unwrap();
    assert_eq!(twist, back);
}
