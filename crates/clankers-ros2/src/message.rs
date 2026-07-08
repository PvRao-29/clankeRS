use async_trait::async_trait;
use serde::Serialize;

/// ROS 2 wire representation a [`RosMessage`] maps to on the `rclrs`/DDS backend.
///
/// The default ([`WireType::StringJson`]) carries the message as a
/// `std_msgs/String` holding its JSON, which works generically for any type. A
/// message that has a native ROS 2 equivalent overrides
/// [`RosMessage::wire_type`] so the backend publishes/subscribes the real typed
/// message (e.g. `sensor_msgs/Image`), giving wire-compatibility with stock ROS
/// nodes (`ros2 topic echo`, Foxglove, `v4l2_camera`, bag play).
///
/// Only the `ros2` backend reads this; the `sim` backend ignores it (everything
/// is JSON on the in-memory bus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    /// `std_msgs/String` envelope holding the message JSON (default transport).
    StringJson,
    /// `sensor_msgs/Image`, via the typed bridge in the `clankers-ros2-dds`
    /// crate (the real DDS backend; see `ros2/clankers-ros2-dds`).
    Image,
}

/// Trait for ROS-compatible messages used by clankeRS.
#[async_trait]
pub trait RosMessage: Clone + Send + Sync + 'static {
    fn topic_type() -> &'static str;

    /// ROS 2 wire type used by the `rclrs`/DDS backend. Defaults to a
    /// `std_msgs/String` JSON envelope; override for types with a native ROS 2
    /// message (see [`WireType`]).
    fn wire_type() -> WireType {
        WireType::StringJson
    }

    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Result<Self, String>
    where
        Self: Sized;
}

/// Simplified sensor_msgs/Image representation.
#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq)]
pub struct ImageMsg {
    pub stamp_nanos: u64,
    pub frame_id: String,
    pub width: u32,
    pub height: u32,
    pub encoding: String,
    pub step: u32,
    pub data: Vec<u8>,
}

impl ImageMsg {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            stamp_nanos: clankers_core::Timestamp::now().as_nanos(),
            frame_id: "camera".to_string(),
            width,
            height,
            encoding: "rgb8".to_string(),
            step: width * 3,
            data,
        }
    }
}

impl RosMessage for ImageMsg {
    fn topic_type() -> &'static str {
        "sensor_msgs/Image"
    }

    /// Published as a real `sensor_msgs/Image` on the `ros2` backend (not a JSON
    /// string), so stock ROS nodes can consume it directly.
    fn wire_type() -> WireType {
        WireType::Image
    }

    fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

/// Single 2D detection.
#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq)]
pub struct Detection {
    pub class_id: u32,
    pub class_name: String,
    pub score: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Array of detections (vision_msgs/Detection2DArray simplified).
#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq)]
pub struct DetectionArray {
    pub stamp_nanos: u64,
    pub frame_id: String,
    pub detections: Vec<Detection>,
}

impl RosMessage for DetectionArray {
    fn topic_type() -> &'static str {
        "clankeRS/DetectionArray"
    }

    fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}
