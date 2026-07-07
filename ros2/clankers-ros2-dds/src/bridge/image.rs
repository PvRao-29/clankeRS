//! `ImageMsg` <-> `sensor_msgs/msg/Image` conversion.
//!
//! The ROS types come from the colcon overlay (`sensor_msgs`, `std_msgs`,
//! `builtin_interfaces`), declared in `package.xml`. `ImageMsg` is the shared
//! type from the [`clankers_ros2`] core crate.

use clankers_ros2::message::ImageMsg;

const NANOS_PER_SEC: u64 = 1_000_000_000;

/// Convert a clankeRS [`ImageMsg`] to a ROS `sensor_msgs/Image`.
pub fn to_ros(msg: &ImageMsg) -> sensor_msgs::msg::Image {
    sensor_msgs::msg::Image {
        header: std_msgs::msg::Header {
            stamp: builtin_interfaces::msg::Time {
                sec: (msg.stamp_nanos / NANOS_PER_SEC) as i32,
                nanosec: (msg.stamp_nanos % NANOS_PER_SEC) as u32,
            },
            frame_id: msg.frame_id.clone(),
        },
        height: msg.height,
        width: msg.width,
        encoding: msg.encoding.clone(),
        is_bigendian: 0,
        step: msg.step,
        data: msg.data.clone(),
    }
}

/// Convert a ROS `sensor_msgs/Image` to a clankeRS [`ImageMsg`].
pub fn from_ros(img: &sensor_msgs::msg::Image) -> ImageMsg {
    let stamp = &img.header.stamp;
    let stamp_nanos = (stamp.sec.max(0) as u64) * NANOS_PER_SEC + stamp.nanosec as u64;
    ImageMsg {
        stamp_nanos,
        frame_id: img.header.frame_id.clone(),
        width: img.width,
        height: img.height,
        encoding: img.encoding.clone(),
        step: img.step,
        data: img.data.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_round_trips_through_ros() {
        let original = ImageMsg {
            stamp_nanos: 1_500_000_042,
            frame_id: "camera_optical".to_string(),
            width: 64,
            height: 48,
            encoding: "rgb8".to_string(),
            step: 64 * 3,
            data: vec![7u8; 64 * 48 * 3],
        };

        let ros = to_ros(&original);
        assert_eq!(ros.width, 64);
        assert_eq!(ros.height, 48);
        assert_eq!(ros.header.stamp.sec, 1);
        assert_eq!(ros.header.stamp.nanosec, 500_000_042);
        assert_eq!(ros.header.frame_id, "camera_optical");

        let back = from_ros(&ros);
        assert_eq!(back, original);
    }
}
