//! Integration tests for the image/pointcloud preprocessing pipeline.

use clankers_ros2::ImageMsg;
use clankers_tensor::{DataLayout, ImageTensor, PointCloudTensor};

fn rgb_msg(width: u32, height: u32, fill: u8) -> ImageMsg {
    ImageMsg {
        stamp_nanos: 0,
        frame_id: "camera".into(),
        width,
        height,
        encoding: "rgb8".into(),
        step: width * 3,
        data: vec![fill; (width * height * 3) as usize],
    }
}

#[test]
fn from_ros_msg_shapes_and_normalizes() {
    let tensor = ImageTensor::from_ros_msg(&rgb_msg(4, 4, 255)).unwrap();
    assert_eq!(tensor.shape(), vec![1, 4, 4, 3]);
    assert_eq!(tensor.layout, DataLayout::Nhwc);
    // 255 / 255 == 1.0
    assert!(tensor.to_vec().iter().all(|&v| (v - 1.0).abs() < 1e-6));
}

#[test]
fn mono_encoding_infers_single_channel() {
    let msg = ImageMsg {
        stamp_nanos: 0,
        frame_id: "cam".into(),
        width: 2,
        height: 2,
        encoding: "mono8".into(),
        step: 2,
        data: vec![0u8; 4],
    };
    let tensor = ImageTensor::from_ros_msg(&msg).unwrap();
    assert_eq!(tensor.shape(), vec![1, 2, 2, 1]);
}

#[test]
fn from_ros_msg_rejects_bad_input() {
    let zero = ImageMsg {
        stamp_nanos: 0,
        frame_id: "cam".into(),
        width: 0,
        height: 4,
        encoding: "rgb8".into(),
        step: 0,
        data: vec![0u8; 4],
    };
    assert!(ImageTensor::from_ros_msg(&zero).is_err());

    let short = ImageMsg {
        stamp_nanos: 0,
        frame_id: "cam".into(),
        width: 4,
        height: 4,
        encoding: "rgb8".into(),
        step: 12,
        data: vec![0u8; 10], // needs 48
    };
    assert!(ImageTensor::from_ros_msg(&short).is_err());
}

#[test]
fn resize_changes_dims_keeps_layout() {
    let tensor = ImageTensor::from_ros_msg(&rgb_msg(8, 8, 128))
        .unwrap()
        .resize(4, 2)
        .unwrap();
    assert_eq!(tensor.width, 4);
    assert_eq!(tensor.height, 2);
    assert_eq!(tensor.shape(), vec![1, 2, 4, 3]);
    assert_eq!(tensor.layout, DataLayout::Nhwc);

    assert!(tensor.clone().resize(0, 4).is_err());
}

#[test]
fn normalize_imagenet_applies_formula() {
    let tensor = ImageTensor::from_ros_msg(&rgb_msg(2, 2, 255))
        .unwrap()
        .normalize_imagenet()
        .unwrap();
    // channel 0: (1.0 - 0.485) / 0.229
    let expected = (1.0f32 - 0.485) / 0.229;
    assert!((tensor.data[[0, 0, 0, 0]] - expected).abs() < 1e-4);
}

#[test]
fn to_nchw_reorders_and_is_idempotent() {
    let nchw = ImageTensor::from_ros_msg(&rgb_msg(5, 3, 200))
        .unwrap()
        .to_nchw()
        .unwrap();
    assert_eq!(nchw.layout, DataLayout::Nchw);
    assert_eq!(nchw.shape(), vec![1, 3, 3, 5]);

    let again = nchw.clone().to_nchw().unwrap();
    assert_eq!(again.shape(), nchw.shape());
}

#[test]
fn pointcloud_from_xyz() {
    let pc = PointCloudTensor::from_xyz(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]).unwrap();
    assert_eq!(pc.num_points(), 2);
    assert_eq!(pc.to_vec(), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}
