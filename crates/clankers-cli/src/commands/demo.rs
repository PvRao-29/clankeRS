use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

/// Run a clankeRS demo. `camera-perception` is the golden-path vertical slice:
/// replay a real MCAP log through preprocess -> ONNX -> detections -> ROS 2
/// publish, and report latency. It delegates to the `camera_replay` example so
/// there is a single canonical implementation.
pub async fn execute(name: &str) -> Result<()> {
    match name {
        "camera-perception" | "camera_perception" => run_example("camera_replay"),
        other => Err(anyhow!("unknown demo: {other}")),
    }
}

fn run_example(example: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args(["run", "--release", "-p", "clankers", "--example", example])
        .current_dir(find_workspace_root()?)
        .status()
        .with_context(|| format!("run example {example}"))?;
    if !status.success() {
        anyhow::bail!("demo '{example}' failed");
    }
    Ok(())
}

fn find_workspace_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    for ancestor in cwd.ancestors() {
        if ancestor.join("Cargo.toml").exists() && ancestor.join("crates").exists() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Ok(cwd)
}
