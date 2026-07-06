use async_trait::async_trait;
use serde::Serialize;

/// Trait for ROS-compatible messages used by clankeRS.
#[async_trait]
pub trait RosMessage: Clone + Send + Sync + 'static {
    fn topic_type() -> &'static str;
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
        "clankers/DetectionArray"
    }

    fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}
