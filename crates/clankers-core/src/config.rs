use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{RobotError, RobotResult};
use crate::types::NodeName;

/// Full clankeRS project configuration from `clankeRS.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClankersConfig {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub node: NodeConfig,
    #[serde(default)]
    pub ros2: Ros2Config,
    #[serde(default)]
    pub topics: TopicsConfig,
    #[serde(default)]
    pub model: HashMap<String, ModelConfig>,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub replay: ReplayConfig,
    #[serde(default)]
    pub testing: TestingConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeConfig {
    pub name: Option<String>,
    #[serde(default = "default_runtime")]
    pub runtime: String,
}

fn default_runtime() -> String {
    "async".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ros2Config {
    #[serde(default)]
    pub domain_id: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicsConfig {
    #[serde(default)]
    pub input: HashMap<String, TopicConfig>,
    #[serde(default)]
    pub output: HashMap<String, TopicConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    pub name: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub qos: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    #[serde(default)]
    pub source_framework: Option<String>,
    pub path: String,
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default = "default_device")]
    pub device: String,
    #[serde(default)]
    pub warmup_runs: Option<u32>,
    #[serde(default)]
    pub max_latency_ms: Option<u64>,
    #[serde(default)]
    pub input: Option<ModelIoConfig>,
    #[serde(default)]
    pub output: Option<ModelIoConfig>,
}

fn default_backend() -> String {
    "onnxruntime".to_string()
}

fn default_device() -> String {
    "cpu".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIoConfig {
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub shape: Option<Vec<usize>>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub dtype: Option<String>,
    #[serde(default)]
    pub normalization: Option<String>,
    #[serde(default)]
    pub validation: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default)]
    pub record_mcap: bool,
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_output_dir() -> String {
    "logs/".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayConfig {
    #[serde(default)]
    pub default_log: Option<String>,
    #[serde(default = "default_true")]
    pub deterministic: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestingConfig {
    #[serde(default)]
    pub max_latency_ms: Option<u64>,
    #[serde(default)]
    pub allow_dropped_messages: bool,
}

impl ClankersConfig {
    pub fn load(path: impl AsRef<Path>) -> RobotResult<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|e| {
            RobotError::Config(format!(
                "failed to read config file '{}': {e}",
                path.display()
            ))
        })?;
        Self::parse(&contents)
    }

    pub fn load_from_dir(dir: impl AsRef<Path>) -> RobotResult<Self> {
        let path = dir.as_ref().join("clankeRS.toml");
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn parse(contents: &str) -> RobotResult<Self> {
        toml::from_str(contents).map_err(|e| {
            RobotError::Config(format!(
                "failed to parse clankeRS.toml: {e}\n\
                 Hint: check section names like [project], [node], [topics.input.camera]"
            ))
        })
    }

    pub fn node_name(&self) -> NodeName {
        NodeName::new(
            self.node
                .name
                .clone()
                .or_else(|| self.project.name.clone())
                .unwrap_or_else(|| "clankers_node".to_string()),
        )
    }

    pub fn model_path(&self, name: &str) -> RobotResult<PathBuf> {
        self.model
            .get(name)
            .map(|m| PathBuf::from(&m.path))
            .ok_or_else(|| RobotError::Model(format!("model '{name}' not found in config")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_example_config() {
        let toml = r#"
[project]
name = "camera_detector"
version = "0.1.0"

[node]
name = "camera_detector"
runtime = "async"

[ros2]
domain_id = 0

[topics.input.camera]
name = "/camera/image_raw"
type = "sensor_msgs/Image"
qos = "sensor_data"

[topics.output.detections]
name = "/detections"
type = "vision_msgs/Detection2DArray"

[model.detector]
source_framework = "pytorch"
path = "models/detector.onnx"
backend = "onnxruntime"
device = "cpu"
warmup_runs = 10
max_latency_ms = 20

[logging]
record_mcap = true
output_dir = "logs/"
level = "info"
"#;
        let config = ClankersConfig::parse(toml).unwrap();
        assert_eq!(config.node_name().as_str(), "camera_detector");
        assert!(config.model.contains_key("detector"));
        assert_eq!(config.topics.input["camera"].name, "/camera/image_raw");
    }
}
