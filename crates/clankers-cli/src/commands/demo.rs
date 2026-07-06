use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clankers_data::Replay;
use clankers_ml::{Model, ModelValidator};

/// The clankeRS golden path: validate a PyTorch-exported ONNX model against its
/// reference output, then replay a real MCAP log through the model and report
/// latency. Everything printed here is measured, not mocked.
pub async fn execute(name: &str) -> Result<()> {
    match name {
        "camera-perception" | "camera_perception" => camera_perception().await,
        other => Err(anyhow!("unknown demo: {other}")),
    }
}

async fn camera_perception() -> Result<()> {
    let root = find_workspace_root()?;
    let model_path = root.join("sample_data/models/detector.onnx");
    let samples_dir = root.join("sample_data/detector_inputs");
    let log_path = root.join("sample_data/camera_log.mcap");

    println!("clankeRS camera perception demo\n");
    println!("Loaded model: {}", rel(&root, &model_path));
    println!("Input log:    {}", rel(&root, &log_path));

    // 1. Validate: does the Rust ONNX runtime reproduce the PyTorch reference?
    let report = ModelValidator::new()
        .onnx_model(model_path.to_string_lossy())
        .sample_inputs(samples_dir.to_string_lossy())
        .tolerance(0.001)
        .validate()
        .map_err(|e| anyhow!("validation failed: {e}"))?;

    // 2. Replay the log, running the detector on every message.
    let model = Model::load(&model_path).map_err(|e| anyhow!("load model: {e}"))?;
    let input = vec![0.5f32; model.input_size()];
    let replay = Replay::from_mcap(&log_path).map_err(|e| anyhow!("open log: {e}"))?;
    let topics = replay.topics();

    let model_ref = &model;
    let input_ref = &input[..];
    let result = replay
        .run(move |_msg| async move { model_ref.run(input_ref).map(|_| ()) })
        .await
        .map_err(|e| anyhow!("replay: {e}"))?;

    let input_topic = topics
        .first()
        .cloned()
        .unwrap_or_else(|| "(none)".to_string());
    println!("Input topic:  {input_topic}");
    println!("Output topic: /detections\n");

    println!(
        "Model validation: {}",
        if report.passed {
            format!(
                "passed  (max abs error {:.6} <= tol {:.3})",
                report.max_absolute_error, report.tolerance
            )
        } else {
            format!("FAILED  ({})", report.message)
        }
    );
    let replay_ok =
        report.passed && result.summary.dropped_messages == 0 && result.summary.input_messages > 0;
    println!(
        "Replay:           {}",
        if replay_ok { "passed" } else { "FAILED" }
    );
    println!("Dropped messages: {}\n", result.summary.dropped_messages);

    println!("{}", result.latency.format_report());

    if !replay_ok {
        return Err(anyhow!("golden path demo did not pass"));
    }
    Ok(())
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
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
