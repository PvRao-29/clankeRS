//! The clankeRS vertical slice, in one command:
//!
//! ```text
//! cargo run -p clankers --example camera_replay
//! ```
//!
//! It replays a real MCAP camera log through the full pipeline —
//! decode frame → preprocess image → ONNX inference → detections →
//! publish to a ROS 2 topic — and reports measured latency. Every number
//! printed is real; nothing here is mocked.

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use clankers::prelude::*;
use clankers::ros2::RosMessage;

const CAMERA_TOPIC: &str = "/camera/image_raw";
const DETECTIONS_TOPIC: &str = "/detections";

#[tokio::main]
async fn main() -> RobotResult<()> {
    let root = workspace_root();
    let model_path = root.join("sample_data/models/detector.onnx");
    let log_path = root.join("sample_data/camera_log.mcap");

    println!("Loading {}...", file_name(&model_path));
    let model = Model::load(&model_path)?;

    println!("Opening {}...", file_name(&log_path));
    let replay = Replay::from_mcap(&log_path)?;
    let frames: Vec<_> = replay
        .messages()
        .iter()
        .filter(|m| m.topic == CAMERA_TOPIC)
        .cloned()
        .collect();
    let total = frames.len();

    // ROS 2 bridge (simulated bus): publish detections and subscribe to prove
    // the messages are actually delivered end-to-end.
    let node = RobotNode::new("camera_replay").await?;
    let detections_pub = node
        .publish::<DetectionArray>(DETECTIONS_TOPIC, QosProfile::default())
        .await?;
    let mut detections_sub = node
        .subscribe::<DetectionArray>(DETECTIONS_TOPIC, QosProfile::default())
        .await?;

    println!("Running replay...\n");
    let tty = std::io::stdout().is_terminal();
    let mut latency = LatencyStats::new();
    let mut published = 0u64;
    let mut received = 0u64;
    let mut dropped = 0u64;
    let wall_start = Instant::now();

    for (i, rec) in frames.iter().enumerate() {
        let start = Instant::now();
        match process_frame(&model, &rec.data) {
            Ok(detections) => {
                // Measure preprocess + inference only (the runtime hot path).
                latency.record(start.elapsed());
                detections_pub
                    .publish(DetectionArray {
                        stamp_nanos: Timestamp::now().as_nanos(),
                        frame_id: "camera".into(),
                        detections,
                    })
                    .await?;
                published += 1;
                // Drain immediately so the bus buffer never grows unbounded.
                if detections_sub.next().await.is_some() {
                    received += 1;
                }
            }
            Err(e) => {
                dropped += 1;
                eprintln!("frame {i} dropped: {e}");
            }
        }
        if tty {
            print!("\rFrame {}/{}", i + 1, total);
            let _ = std::io::stdout().flush();
        } else if (i + 1) % 50 == 0 || i + 1 == total {
            println!("Frame {}/{}", i + 1, total);
        }
    }
    if tty {
        println!();
    }
    let wall = wall_start.elapsed();

    println!("\nPublished {published} detections to {DETECTIONS_TOPIC}");
    println!("Replay complete.\n");

    let fps = if wall.as_secs_f64() > 0.0 {
        published as f64 / wall.as_secs_f64()
    } else {
        0.0
    };
    println!("Replay Summary");
    println!("  Frames:    {total}");
    println!("  FPS:       {fps:.1}");
    println!("  Detections received on {DETECTIONS_TOPIC}: {received}");
    println!("  Dropped:   {dropped}\n");
    println!("{}\n", latency.format_report());

    let passed = dropped == 0 && published as usize == total && received == published;
    if passed {
        println!("✓ Replay passed");
        Ok(())
    } else {
        println!("✗ Replay failed");
        Err(RobotError::Other("replay did not pass".into()))
    }
}

/// Decode one camera frame and run it through the full inference pipeline.
fn process_frame(model: &Model, bytes: &[u8]) -> RobotResult<Vec<Detection>> {
    let image = ImageMsg::deserialize(bytes).map_err(RobotError::Other)?;
    let tensor = ImageTensor::from_ros_msg(&image)?
        .resize(224, 224)?
        .normalize_imagenet()?
        .to_nchw()?;
    let output = model.run(&tensor.to_vec())?;
    Ok(top_detection(&output))
}

/// Turn raw model logits into the single highest-scoring detection.
fn top_detection(output: &[f32]) -> Vec<Detection> {
    let mut best = 0usize;
    for (i, &v) in output.iter().enumerate() {
        if v > output[best] {
            best = i;
        }
    }
    vec![Detection {
        class_id: best as u32,
        class_name: format!("class_{best}"),
        score: output.get(best).copied().unwrap_or(0.0),
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    }]
}

fn workspace_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for ancestor in cwd.ancestors() {
        if ancestor.join("sample_data/models/detector.onnx").exists() {
            return ancestor.to_path_buf();
        }
    }
    cwd
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}
