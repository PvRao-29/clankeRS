use std::path::PathBuf;
use std::sync::Arc;

use crate::config::ClankeRSConfig;
use crate::error::RobotResult;
use crate::types::NodeName;

/// Runtime context for a clankeRS node.
#[derive(Clone)]
pub struct RobotContext {
    pub config: Arc<ClankeRSConfig>,
    pub work_dir: PathBuf,
}

impl RobotContext {
    pub fn new(config: ClankeRSConfig, work_dir: impl Into<PathBuf>) -> Self {
        Self {
            config: Arc::new(config),
            work_dir: work_dir.into(),
        }
    }

    pub fn from_config_file(path: impl Into<PathBuf>) -> RobotResult<Self> {
        let path = path.into();
        let work_dir = path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let config = ClankeRSConfig::load(&path)?;
        Ok(Self::new(config, work_dir))
    }

    pub fn from_work_dir(work_dir: impl Into<PathBuf>) -> RobotResult<Self> {
        let work_dir = work_dir.into();
        let config = ClankeRSConfig::load_from_dir(&work_dir)?;
        Ok(Self::new(config, work_dir))
    }

    pub fn node_name(&self) -> NodeName {
        self.config.node_name()
    }

    pub fn model_config(&self, name: &str) -> RobotResult<&crate::config::ModelConfig> {
        self.config.model.get(name).ok_or_else(|| {
            crate::error::RobotError::Model(format!("model '{name}' not configured"))
        })
    }

    pub fn resolve_path(&self, relative: impl AsRef<std::path::Path>) -> PathBuf {
        self.work_dir.join(relative)
    }

    /// Load a model by name from config (requires `clankers-ml` at call site).
    pub fn model_path(&self, name: &str) -> RobotResult<PathBuf> {
        self.config.model_path(name).map(|p| self.resolve_path(p))
    }
}
