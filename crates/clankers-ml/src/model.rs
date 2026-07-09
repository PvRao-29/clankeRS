//! The main clankeRS inference API.
//!
//! [`Model`] is the user-facing optimized inference runtime. It loads a model
//! through a pluggable backend (ONNX Runtime by default), binds zero-copy
//! [`TensorView`] inputs, and returns named outputs with per-run profiling.
//!
//! [`crate::inference::InferenceEngine`] is the lower-level building block used
//! internally and by power users implementing custom backends or allocation
//! policies.

use std::path::{Path, PathBuf};
use std::time::Duration;

use clankers_core::config::{ModelBackendKind, ModelConfig};
use clankers_core::{LatencyStats, RobotError, RobotResult};
use clankers_tensor::{AllocationPolicy, Device, Shape, TensorView, TensorViewMut};

use crate::backend::NoopBackend;
use crate::inference::{InferenceEngine, InferenceStats, ModelEngine, ModelSource, NamedOutputs};

#[cfg(feature = "onnxruntime")]
use crate::backend::OnnxRuntimeBackend;

/// Backend selection for [`ModelBuilder::backend`].
pub enum RuntimeBackend {
    #[cfg(feature = "onnxruntime")]
    OnnxRuntime(OnnxRuntimeBackend),
    Noop(NoopBackend),
}

#[cfg(feature = "onnxruntime")]
impl Default for RuntimeBackend {
    fn default() -> Self {
        RuntimeBackend::OnnxRuntime(OnnxRuntimeBackend::default())
    }
}

#[cfg(not(feature = "onnxruntime"))]
impl Default for RuntimeBackend {
    fn default() -> Self {
        RuntimeBackend::Noop(NoopBackend::identity(
            clankers_tensor::ShapeSpec::from_onnx_dims(&[1, 1]),
        ))
    }
}

#[cfg(feature = "onnxruntime")]
impl From<OnnxRuntimeBackend> for RuntimeBackend {
    fn from(backend: OnnxRuntimeBackend) -> Self {
        RuntimeBackend::OnnxRuntime(backend)
    }
}

impl From<NoopBackend> for RuntimeBackend {
    fn from(backend: NoopBackend) -> Self {
        RuntimeBackend::Noop(backend)
    }
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub path: PathBuf,
    pub backend: String,
    pub source_framework: Option<String>,
    pub input_shapes: Vec<Vec<usize>>,
    pub output_shapes: Vec<Vec<usize>>,
}

/// A loaded clankeRS model backed by the optimized inference runtime.
///
/// `Model` is the main API most users should use. It supports simple
/// single-input inference, named multi-input inference, zero-copy tensor views,
/// backend selection, profiling, and preallocated output buffers when supported
/// by the backend.
pub struct Model {
    engine: ModelEngine,
    metadata: ModelMetadata,
    last_stats: Option<InferenceStats>,
    max_latency: Option<Duration>,
    latency_stats: LatencyStats,
    primary_input_shape: Shape,
}

pub struct ModelBuilder {
    path: Option<PathBuf>,
    backend: RuntimeBackend,
    metadata_backend: Option<String>,
    source_framework: Option<String>,
    device: Device,
    warmup_runs: u32,
    max_latency: Option<Duration>,
    allocation_policy: AllocationPolicy,
    strict_realtime: bool,
}

impl Model {
    pub fn load(path: impl AsRef<Path>) -> RobotResult<Self> {
        Self::builder().load(path)
    }

    pub fn builder() -> ModelBuilder {
        ModelBuilder::new()
    }

    pub fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    /// Access the underlying optimized runtime (advanced).
    pub fn engine(&self) -> &ModelEngine {
        &self.engine
    }

    /// Mutable access to the underlying runtime (advanced).
    pub fn engine_mut(&mut self) -> &mut ModelEngine {
        &mut self.engine
    }

    /// Stats from the most recent inference call, if any.
    pub fn stats(&self) -> Option<InferenceStats> {
        self.last_stats
    }

    /// Cumulative wall-clock latency stats across [`run`](Self::run) calls.
    pub fn latency_stats(&self) -> LatencyStats {
        self.latency_stats.clone()
    }

    pub fn input_size(&self) -> usize {
        self.metadata
            .input_shapes
            .first()
            .map(|s| s.iter().product())
            .unwrap_or(0)
    }

    /// Convenience single-input inference over a flat `f32` buffer.
    ///
    /// For multi-input models or zero-copy sensor buffers, prefer
    /// [`run_named`](Self::run_named).
    pub fn run(&mut self, input: &[f32]) -> RobotResult<Vec<f32>> {
        let shape = self.primary_input_shape.clone();
        let view = TensorView::from_f32(input, &shape).map_err(RobotError::from)?;
        let spec_name = self
            .engine
            .input_specs()
            .first()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "input".to_string());
        let outputs = self.run_named([(spec_name.as_str(), view)])?;
        outputs
            .first()
            .ok_or_else(|| RobotError::Model("model produced no outputs".into()))?
            .to_f32_vec()
            .map_err(RobotError::from)
    }

    /// Run inference with named, zero-copy tensor inputs.
    pub fn run_named<'a, I>(&mut self, inputs: I) -> RobotResult<NamedOutputs>
    where
        I: IntoIterator<Item = (&'a str, TensorView<'a>)>,
    {
        let named: Vec<_> = inputs.into_iter().collect();
        let (outputs, stats) = self
            .engine
            .run_named_with_stats(&named)
            .map_err(RobotError::from)?;
        self.last_stats = Some(stats);
        self.record_latency(stats.latency);
        Ok(outputs)
    }

    /// Run inference writing outputs into caller-preallocated buffers.
    ///
    /// Returns an error when the backend does not support preallocated outputs
    /// (ONNX Runtime today copies outputs out of the runtime).
    pub fn run_into<'a, I>(
        &mut self,
        inputs: I,
        outputs: &mut [TensorViewMut<'a>],
    ) -> RobotResult<InferenceStats>
    where
        I: IntoIterator<Item = (&'a str, TensorView<'a>)>,
    {
        let named: Vec<_> = inputs.into_iter().collect();
        let stats = self
            .engine
            .run_into(&named, outputs)
            .map_err(RobotError::from)?;
        self.last_stats = Some(stats);
        self.record_latency(stats.latency);
        Ok(stats)
    }

    fn record_latency(&mut self, elapsed: Duration) {
        if let Some(max) = self.max_latency {
            if elapsed > max {
                tracing::warn!(
                    elapsed_ms = elapsed.as_secs_f64() * 1000.0,
                    max_ms = max.as_secs_f64() * 1000.0,
                    "inference exceeded max latency"
                );
            }
        }
        self.latency_stats.record(elapsed);
    }
}

impl ModelBuilder {
    fn new() -> Self {
        ModelBuilder {
            path: None,
            backend: RuntimeBackend::default(),
            metadata_backend: None,
            source_framework: None,
            device: Device::Cpu,
            warmup_runs: 0,
            max_latency: None,
            allocation_policy: AllocationPolicy::Dynamic,
            strict_realtime: false,
        }
    }

    /// Start a builder from a [`ModelConfig`] and resolved model path.
    pub fn from_config(config: &ModelConfig, path: impl AsRef<Path>) -> RobotResult<Self> {
        match config.backend_kind()? {
            ModelBackendKind::Noop => {
                return Err(RobotError::Model(
                    "noop backend is not supported for Model; use InferenceEngine directly".into(),
                ));
            }
            ModelBackendKind::OnnxRuntime => {}
        }

        #[cfg(not(feature = "onnxruntime"))]
        {
            let _ = path.as_ref();
            return Err(RobotError::Model(
                "onnxruntime backend requires the onnxruntime feature".into(),
            ));
        }

        #[cfg(feature = "onnxruntime")]
        {
            let device = Device::parse(&config.device).ok_or_else(|| {
                RobotError::Model(format!("unsupported model device {:?}", config.device))
            })?;
            let mut builder = Self::new()
                .path(path)
                .backend(RuntimeBackend::OnnxRuntime(OnnxRuntimeBackend::default()))
                .metadata_backend(config.backend.clone())
                .device(device)
                .source_framework(config.source_framework.clone().unwrap_or_default());
            if let Some(warmup) = config.warmup_runs {
                builder = builder.warmup_runs(warmup);
            }
            if let Some(max) = config.max_latency() {
                builder = builder.max_latency(max);
            }
            Ok(builder)
        }
    }

    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Load from `path` and build the model (alias for `.path(path).build()`).
    pub fn load(self, path: impl AsRef<Path>) -> RobotResult<Model> {
        self.path(path).build()
    }

    pub fn backend(mut self, backend: impl Into<RuntimeBackend>) -> Self {
        self.backend = backend.into();
        self
    }

    fn metadata_backend(mut self, backend: impl Into<String>) -> Self {
        self.metadata_backend = Some(backend.into());
        self
    }

    pub fn device(mut self, device: Device) -> Self {
        self.device = device;
        self
    }

    pub fn source_framework(mut self, framework: impl Into<String>) -> Self {
        let value = framework.into();
        if !value.is_empty() {
            self.source_framework = Some(value);
        }
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

    pub fn allocation_policy(mut self, policy: AllocationPolicy) -> Self {
        self.allocation_policy = policy;
        self
    }

    pub fn strict_realtime(mut self, strict: bool) -> Self {
        self.strict_realtime = strict;
        self
    }

    pub fn build(self) -> RobotResult<Model> {
        let path = self
            .path
            .clone()
            .ok_or_else(|| RobotError::Model("model path not set".into()))?;

        let engine = match self.backend {
            #[cfg(feature = "onnxruntime")]
            RuntimeBackend::OnnxRuntime(backend) => ModelEngine::from(
                InferenceEngine::builder(backend)
                    .model(ModelSource::Path(path.clone()))
                    .device(self.device)
                    .warmup(self.warmup_runs)
                    .allocation_policy(self.allocation_policy)
                    .strict_realtime(self.strict_realtime)
                    .build()
                    .map_err(RobotError::from)?,
            ),
            RuntimeBackend::Noop(backend) => ModelEngine::from(
                InferenceEngine::builder(backend)
                    .model(ModelSource::Path(path.clone()))
                    .device(self.device)
                    .warmup(self.warmup_runs)
                    .allocation_policy(self.allocation_policy)
                    .strict_realtime(self.strict_realtime)
                    .build()
                    .map_err(RobotError::from)?,
            ),
        };

        let input_shapes = specs_to_dims(engine.input_specs());
        let output_shapes = specs_to_dims(engine.output_specs());
        let primary_input_shape = engine
            .input_specs()
            .first()
            .map(|s| s.shape.concrete_or_unit())
            .unwrap_or_else(Shape::scalar);

        let backend_name = self
            .metadata_backend
            .unwrap_or_else(|| engine.backend_name().to_string());

        Ok(Model {
            engine,
            metadata: ModelMetadata {
                path,
                backend: backend_name,
                source_framework: self.source_framework,
                input_shapes,
                output_shapes,
            },
            last_stats: None,
            max_latency: self.max_latency,
            latency_stats: LatencyStats::new(),
            primary_input_shape,
        })
    }
}

fn specs_to_dims(specs: &[crate::backend::TensorSpec]) -> Vec<Vec<usize>> {
    specs
        .iter()
        .map(|s| s.shape.concrete_or_unit().dims().to_vec())
        .collect()
}
