//! Integration tests for config parsing and the RobotContext path helpers.

use clankers_core::{ClankeRSConfig, RobotContext};

const SAMPLE: &str = r#"
[project]
name = "test_proj"
version = "0.1.0"

[node]
name = "perception"

[model.detector]
path = "models/detector.onnx"
"#;

#[test]
fn parse_and_lookup() {
    let cfg = ClankeRSConfig::parse(SAMPLE).unwrap();
    // node.name wins over project.name.
    assert_eq!(cfg.node_name().as_str(), "perception");
    assert_eq!(
        cfg.model_path("detector").unwrap().to_str().unwrap(),
        "models/detector.onnx"
    );
    assert!(cfg.model_path("missing").is_err());
}

#[test]
fn node_name_falls_back_to_project() {
    let cfg = ClankeRSConfig::parse("[project]\nname = \"only_project\"\n").unwrap();
    assert_eq!(cfg.node_name().as_str(), "only_project");
}

#[test]
fn parse_rejects_invalid_toml() {
    assert!(ClankeRSConfig::parse("this is = = not toml").is_err());
}

#[test]
fn load_from_dir_defaults_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    // No clankeRS.toml present -> default config, default node name.
    let cfg = ClankeRSConfig::load_from_dir(dir.path()).unwrap();
    assert_eq!(cfg.node_name().as_str(), "clankers_node");
}

#[test]
fn context_resolves_paths_against_work_dir() {
    let cfg = ClankeRSConfig::parse(SAMPLE).unwrap();
    let ctx = RobotContext::new(cfg, "/tmp/robot");

    assert_eq!(ctx.node_name().as_str(), "perception");
    assert_eq!(
        ctx.resolve_path("logs/run.mcap"),
        std::path::Path::new("/tmp/robot/logs/run.mcap")
    );
    assert_eq!(
        ctx.model_path("detector").unwrap(),
        std::path::Path::new("/tmp/robot/models/detector.onnx")
    );
    assert!(ctx.model_config("detector").is_ok());
    assert!(ctx.model_config("missing").is_err());
}

#[test]
fn context_from_work_dir_reads_config_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("clankeRS.toml"), SAMPLE).unwrap();

    let ctx = RobotContext::from_work_dir(dir.path()).unwrap();
    assert_eq!(ctx.node_name().as_str(), "perception");
}
