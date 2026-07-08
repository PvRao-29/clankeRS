//! End-to-end integration tests exercising the public `clankers` facade the way
//! a downstream node would: MCAP write -> replay -> preprocess -> sim pub/sub,
//! plus an ONNX inference wire-up gated on the default `ml` feature.

use clankers::data::McapWriter;
use clankers::prelude::*;
use clankers::ros2::RosMessage;

const TOPIC_IMAGE: &str = "/camera/image_raw";
const TOPIC_DET: &str = "/e2e/detections";
const FRAMES: usize = 5;

fn write_image_log(path: &std::path::Path) {
    let mut w = McapWriter::create(path).unwrap();
    for i in 0..FRAMES as u64 {
        let img = ImageMsg::new(8, 8, vec![(i * 10) as u8; 8 * 8 * 3]);
        w.write_message(
            TOPIC_IMAGE,
            "sensor_msgs/Image",
            "json",
            &img.serialize(),
            Timestamp::from_nanos(i * 33_000_000),
        )
        .unwrap();
    }
    w.finish().unwrap();
}

#[tokio::test]
async fn replay_preprocess_publish_pipeline() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("camera.mcap");
    write_image_log(&path);

    let node = RobotNode::new("e2e").await.unwrap();
    let mut detections = node
        .subscribe::<DetectionArray>(TOPIC_DET, QosProfile::default())
        .await
        .unwrap();
    let publisher = node
        .publish::<DetectionArray>(TOPIC_DET, QosProfile::default())
        .await
        .unwrap();

    let replay = Replay::from_mcap(&path).unwrap();
    let result = replay
        .run(|msg| {
            let publisher = &publisher;
            async move {
                // Decode the recorded image and run the preprocessing pipeline.
                let image = ImageMsg::deserialize(&msg.data).map_err(RobotError::Other)?;
                let _tensor = ImageTensor::from_ros_msg(&image)?
                    .resize(4, 4)?
                    .normalize_imagenet()?
                    .to_nchw()?;

                publisher
                    .publish(DetectionArray {
                        stamp_nanos: image.stamp_nanos,
                        frame_id: image.frame_id.clone(),
                        detections: vec![Detection {
                            class_id: 0,
                            class_name: "object".into(),
                            score: 1.0,
                            x: 0.0,
                            y: 0.0,
                            width: 1.0,
                            height: 1.0,
                        }],
                    })
                    .await?;
                Ok(())
            }
        })
        .await
        .unwrap();

    assert_eq!(result.summary.input_messages, FRAMES as u64);
    assert_eq!(result.summary.dropped_messages, 0);

    // Every published detection is delivered on the sim bus.
    for _ in 0..FRAMES {
        let det = detections.next().await.unwrap();
        assert_eq!(det.detections.len(), 1);
    }
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn onnx_inference_through_facade() {
    let model_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../sample_data/models/detector.onnx");
    let model = Model::load(&model_path).unwrap();
    let output = model.run(&vec![0.5f32; model.input_size()]).unwrap();
    assert!(!output.is_empty());
}
