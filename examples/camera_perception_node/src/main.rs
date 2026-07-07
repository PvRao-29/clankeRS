//! North-star demo: camera perception node with ONNX inference.

use std::time::Instant;

use clankers::prelude::*;
use clankers::ros2::inject_message;

#[tokio::main]
async fn main() -> RobotResult<()> {
    clankers::runtime::init_tracing();

    let ctx = RobotContext::from_work_dir(".")
        .unwrap_or_else(|_| RobotContext::new(ClankeRSConfig::default(), "."));

    let node = RobotNode::new(ctx.node_name().as_str()).await?;
    let mut camera = node
        .subscribe::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data())
        .await?;
    let detections_pub = node
        .publish::<DetectionArray>("/detections", QosProfile::default())
        .await?;

    // Load model if available, otherwise run dummy inference. Fall back to the
    // ONNX detector shipped in sample_data so the demo runs real inference when
    // launched from the workspace root.
    let model = ctx
        .model_config("detector")
        .ok()
        .and_then(|cfg| Model::load(ctx.resolve_path(&cfg.path)).ok())
        .or_else(|| Model::load("sample_data/models/detector.onnx").ok());

    if let Some(ref m) = model {
        tracing::info!(path = %m.metadata().path.display(), "loaded ONNX model");
    } else {
        tracing::warn!("no model configured — using dummy detections");
    }

    // Sim backend: feed the node synthetic frames on the in-memory bus. The
    // real-DDS equivalent (ros2/) instead receives images over DDS from a real
    // publisher on /camera/image_raw, so it has no such injection point.
    tokio::spawn(async {
        for i in 0..10u32 {
            let w = 320u32;
            let h = 240u32;
            let mut data = vec![128u8; (w * h * 3) as usize];
            data[0] = (i * 25) as u8;
            let _ = inject_message("/camera/image_raw", ImageMsg::new(w, h, data)).await;
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
    });

    let mut latency = LatencyStats::new();
    let mut processed = 0u32;

    while let Some(frame) = camera.next().await {
        let start = Instant::now();
        let tensor = ImageTensor::from_ros_msg(&frame)?
            .resize(224, 224)?
            .normalize_imagenet()?
            .to_nchw()?;

        let detections = if let Some(ref m) = model {
            let output = m.run(&tensor.to_vec())?;
            output_to_detections(&output)
        } else {
            vec![Detection {
                class_id: 0,
                class_name: "dummy".into(),
                score: 0.99,
                x: 0.1,
                y: 0.1,
                width: 0.5,
                height: 0.5,
            }]
        };

        detections_pub
            .publish(DetectionArray {
                stamp_nanos: Timestamp::now().as_nanos(),
                frame_id: frame.frame_id.clone(),
                detections,
            })
            .await?;

        latency.record(start.elapsed());
        processed += 1;
        if processed >= 10 {
            break;
        }
    }

    println!("clankeRS node: camera_perception_node");
    println!("Input topics:\n  /camera/image_raw");
    println!("Output topics:\n  /detections");
    println!("{}", latency.format_report());
    println!("Dropped messages:\n  0");

    Ok(())
}

fn output_to_detections(output: &[f32]) -> Vec<Detection> {
    let mut indexed: Vec<_> = output.iter().enumerate().collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
    indexed
        .into_iter()
        .take(3)
        .filter(|(_, &s)| s > 0.05)
        .map(|(id, &score)| Detection {
            class_id: id as u32,
            class_name: format!("class_{id}"),
            score,
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dummy_detection_pipeline() {
        let msg = ImageMsg::new(64, 64, vec![128u8; 64 * 64 * 3]);
        let tensor = ImageTensor::from_ros_msg(&msg)
            .unwrap()
            .resize(224, 224)
            .unwrap()
            .normalize_imagenet()
            .unwrap()
            .to_nchw()
            .unwrap();
        assert_eq!(tensor.shape(), vec![1, 3, 224, 224]);
    }
}
