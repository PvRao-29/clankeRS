//! Minimal real-DDS publisher.
//!
//! Publishes a `sensor_msgs/msg/Image` on `/camera/image_raw` at ~5 Hz over a
//! real ROS 2 graph, using the rclrs/DDS backend. This is the target for the
//! end-to-end smoke test: from a second sourced shell,
//!
//! ```text
//! ros2 topic type /camera/image_raw   # -> sensor_msgs/msg/Image
//! ros2 topic echo /camera/image_raw   # -> typed header/height/width/encoding/step/data
//! ```
//!
//! Ctrl-C returns cleanly, which drops the node and halts+joins the rclrs
//! executor thread (no teardown warning). See `ros2/README.md`.

use std::time::Duration;

use clankers_ros2_dds::{ImageMsg, QosProfile, RobotNode, RobotResult};

#[tokio::main]
async fn main() -> RobotResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let node = RobotNode::new("pubsub_minimal_dds").await?;
    let publisher = node
        .publish::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data())
        .await?;

    tracing::info!("publishing sensor_msgs/Image on /camera/image_raw (Ctrl-C to stop)");

    let mut frame = 0u32;
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!(frames = frame, "shutdown requested; stopping publisher");
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(200)) => {
                let (w, h) = (64u32, 64u32);
                // A gradient that changes per frame, so `ros2 topic echo` shows
                // the data field moving.
                let data = vec![((frame.wrapping_mul(20)) % 256) as u8; (w * h * 3) as usize];
                publisher.publish(ImageMsg::new(w, h, data)).await?;
                tracing::info!(frame, "published /camera/image_raw");
                frame += 1;
            }
        }
    }

    // Returning drops `node`, which halts and joins the rclrs executor thread
    // before the DDS context is torn down.
    Ok(())
}
