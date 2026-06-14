use std::path::Path;

use clankers_core::{RobotResult, Timestamp};
use clankers_ros2::{ImageMsg, RosMessage};

use crate::writer::McapWriter;

/// Generate a small sample camera MCAP for tests and demos.
pub fn generate_camera_log(path: impl AsRef<Path>) -> RobotResult<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut writer = McapWriter::create(path)?;
    let num_frames = 200u32;
    for i in 0..num_frames {
        let w = 64u32;
        let h = 64u32;
        let mut data = vec![100u8; (w * h * 3) as usize];
        data[0] = (i % 255) as u8;
        let msg = ImageMsg::new(w, h, data);
        let bytes = msg.serialize();
        let ts = Timestamp::from_nanos(i as u64 * 33_000_000);
        writer.write_message("/camera/image_raw", "sensor_msgs/Image", "json", &bytes, ts)?;
    }
    writer.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::McapLog;

    #[test]
    fn generate_and_inspect_sample() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("camera_log.mcap");
        generate_camera_log(&path).unwrap();
        let log = McapLog::open(&path).unwrap();
        assert!(!log.report().topics.is_empty());
    }

    #[test]
    fn generate_workspace_sample_data() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = workspace.join("sample_data/camera_log.mcap");
        if path.exists() {
            return;
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        generate_camera_log(&path).expect("generate sample_data/camera_log.mcap");
    }
}
