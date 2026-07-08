//! Tests for the transport-agnostic message types and QoS profiles.

use clankers_ros2::qos::{Durability, Reliability};
use clankers_ros2::{Detection, DetectionArray, ImageMsg, QosProfile, RosMessage, WireType};

#[test]
fn image_msg_defaults() {
    let msg = ImageMsg::new(64, 48, vec![0u8; 64 * 48 * 3]);
    assert_eq!(msg.width, 64);
    assert_eq!(msg.height, 48);
    assert_eq!(msg.encoding, "rgb8");
    assert_eq!(msg.frame_id, "camera");
    assert_eq!(msg.step, 64 * 3);
}

#[test]
fn image_msg_round_trip() {
    let msg = ImageMsg::new(4, 4, vec![7u8; 48]);
    let bytes = msg.serialize();
    let back = ImageMsg::deserialize(&bytes).unwrap();
    assert_eq!(msg, back);

    assert_eq!(ImageMsg::topic_type(), "sensor_msgs/Image");
    assert_eq!(ImageMsg::wire_type(), WireType::Image);
}

#[test]
fn detection_array_round_trip() {
    let msg = DetectionArray {
        stamp_nanos: 42,
        frame_id: "camera".into(),
        detections: vec![Detection {
            class_id: 1,
            class_name: "person".into(),
            score: 0.9,
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        }],
    };
    let back = DetectionArray::deserialize(&msg.serialize()).unwrap();
    assert_eq!(msg, back);

    assert_eq!(DetectionArray::topic_type(), "clankeRS/DetectionArray");
    // Default wire type is the JSON envelope.
    assert_eq!(DetectionArray::wire_type(), WireType::StringJson);
}

#[test]
fn deserialize_garbage_errors() {
    assert!(ImageMsg::deserialize(b"not json at all").is_err());
}

#[test]
fn qos_profiles() {
    let def = QosProfile::default();
    assert_eq!(def.reliability, Reliability::Reliable);
    assert_eq!(def.durability, Durability::Volatile);
    assert_eq!(def.history_depth, 10);

    let sensor = QosProfile::sensor_data();
    assert_eq!(sensor.reliability, Reliability::BestEffort);
    assert_eq!(sensor.history_depth, 5);
}
