use std::path::Path;
use std::time::Duration;

use clankers_core::{RobotError, RobotResult};

use crate::backend::{load_onnx, BackendKind, ModelBackend};

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub path: std::path::PathBuf,
    pub backend: String,
    pub source_framework: Option<String>,
    pub input_shapes: Vec<Vec<usize>>,
    pub output_shapes: Vec<Vec<usize>>,
}

pub struct Model {
    backend: Box<dyn ModelBackend>,
    metadata: ModelMetadata,
    warmup_runs: u32,
    max_latency: Option<Duration>,
    latency_stats: std::sync::Mutex<clankers_core::LatencyStats>,
}

pub struct ModelBuilder {
    path: Option<std::path::PathBuf>,
    backend: BackendKind,
    source_framework: Option<String>,
    warmup_runs: u32,
    max_latency: Option<Duration>,
}

impl Model {
    pub fn load(path: impl AsRef<Path>) -> RobotResult<Self> {
        Self::builder().path(path).build()
    }

    pub fn builder() -> ModelBuilder {
        ModelBuilder {
            path: None,
            backend: BackendKind::OnnxRuntime,
            source_framework: None,
            warmup_runs: 0,
            max_latency: None,
        }
    }

    pub fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    pub fn run(&self, input: &[f32]) -> RobotResult<Vec<f32>> {
        let (output, elapsed) = self.backend.run_with_latency(input)?;
        if let Some(max) = self.max_latency {
            if elapsed > max {
                tracing::warn!(
                    elapsed_ms = elapsed.as_secs_f64() * 1000.0,
                    max_ms = max.as_secs_f64() * 1000.0,
                    "inference exceeded max latency"
                );
            }
        }
        self.latency_stats.lock().unwrap().record(elapsed);
        Ok(output)
    }

    pub fn latency_stats(&self) -> clankers_core::LatencyStats {
        self.latency_stats.lock().unwrap().clone()
    }

    pub fn input_size(&self) -> usize {
        self.metadata
            .input_shapes
            .first()
            .map(|s| s.iter().product())
            .unwrap_or(0)
    }
}

impl ModelBuilder {
    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn backend(mut self, backend: BackendKind) -> Self {
        self.backend = backend;
        self
    }

    pub fn source_framework(mut self, framework: impl Into<String>) -> Self {
        self.source_framework = Some(framework.into());
        self
    }

    pub fn warmup_runs(mut self, runs: u32) -> Self {
        self.warmup_runs = runs;
        self
    }

    pub fn max_latency(mut self, duration: Duration) -> Self {
        self.max_latency = Some(duration);
        self
    }

    pub fn build(self) -> RobotResult<Model> {
        let path = self
            .path
            .ok_or_else(|| RobotError::Model("model path not set".into()))?;

        let backend: Box<dyn ModelBackend> = match self.backend {
            BackendKind::OnnxRuntime => load_onnx(&path)?,
        };

        let metadata = ModelMetadata {
            path: path.clone(),
            backend: self.backend.as_str().to_string(),
            source_framework: self.source_framework,
            input_shapes: backend.input_shapes(),
            output_shapes: backend.output_shapes(),
        };

        let model = Model {
            backend,
            metadata,
            warmup_runs: self.warmup_runs,
            max_latency: self.max_latency,
            latency_stats: std::sync::Mutex::new(clankers_core::LatencyStats::new()),
        };

        if model.warmup_runs > 0 {
            let size = model.input_size();
            let dummy = vec![0.0f32; size];
            for _ in 0..model.warmup_runs {
                let _ = model.run(&dummy);
            }
        }

        Ok(model)
    }
}
