//! The engine builder.
//!
//! [`InferenceEngineBuilder`] turns an [`InferenceBackend`] plus a
//! [`ModelSource`] into a ready-to-run [`InferenceEngine`]. It performs the
//! build-time negotiation the engine relies on at run time: it checks the
//! requested [`Device`] against the backend's capabilities and, if `warmup` runs
//! were requested, primes the model with zero-filled inputs so the first real
//! frame doesn't pay one-off costs.

use clankers_tensor::{AllocationPolicy, Device, Tensor, TensorArena};

use crate::backend::InferenceBackend;
use crate::inference::engine::InferenceEngine;
use crate::inference::{InferenceError, InferenceResult, ModelSource};

/// Builds an [`InferenceEngine`] around a backend.
pub struct InferenceEngineBuilder<B: InferenceBackend> {
    backend: B,
    source: ModelSource,
    policy: AllocationPolicy,
    device: Device,
    warmup: u32,
    strict_realtime: bool,
}

impl<B: InferenceBackend> InferenceEngineBuilder<B> {
    /// Start a builder around `backend`. Defaults: no model source, dynamic
    /// allocation, CPU, no warmup, no real-time guarantee.
    pub fn new(backend: B) -> Self {
        InferenceEngineBuilder {
            backend,
            source: ModelSource::None,
            policy: AllocationPolicy::Dynamic,
            device: Device::Cpu,
            warmup: 0,
            strict_realtime: false,
        }
    }

    /// The model to load (a path, in-memory bytes, or `None` for self-contained
    /// backends like [`NoopBackend`](crate::backend::NoopBackend)).
    pub fn model(mut self, source: impl Into<ModelSource>) -> Self {
        self.source = source.into();
        self
    }

    /// The allocation policy for the engine's arena.
    pub fn allocation_policy(mut self, policy: AllocationPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The device to run on. Rejected at build time if the backend can't run there.
    pub fn device(mut self, device: Device) -> Self {
        self.device = device;
        self
    }

    /// Number of zero-input warm-up runs to perform at build time.
    pub fn warmup(mut self, runs: u32) -> Self {
        self.warmup = runs;
        self
    }

    /// Apply the common fields of a [`ModelConfig`] — `warmup_runs` and `device`
    /// — to this builder, returning an error for an unparseable/unsupported
    /// device. The backend itself is chosen by the caller (via which backend the
    /// builder was constructed with), matching `config.backend`.
    ///
    /// [`ModelConfig`]: clankers_core::config::ModelConfig
    pub fn apply_model_config(
        mut self,
        config: &clankers_core::config::ModelConfig,
    ) -> InferenceResult<Self> {
        if let Some(warmup) = config.warmup_runs {
            self.warmup = warmup;
        }
        self.device = Device::parse(&config.device)
            .ok_or_else(|| InferenceError::UnsupportedDevice(config.device.clone()))?;
        Ok(self)
    }

    /// Require that the backend can satisfy the no-allocation `run_into` hot loop.
    ///
    /// When `true`, [`build`](Self::build) fails unless the backend advertises
    /// both zero-copy inputs and preallocated outputs — turning a latent
    /// real-time violation into a build-time error.
    pub fn strict_realtime(mut self, strict: bool) -> Self {
        self.strict_realtime = strict;
        self
    }

    /// Load the model and build the engine.
    pub fn build(self) -> InferenceResult<InferenceEngine<B::Session>> {
        let name = self.backend.name().to_string();
        let capabilities = self.backend.capabilities();

        if !capabilities.accepts_device(self.device) {
            return Err(InferenceError::UnsupportedDevice(format!(
                "backend {name:?} does not support {}",
                self.device
            )));
        }

        if self.strict_realtime {
            if !capabilities.supports_preallocated_outputs {
                return Err(InferenceError::RealtimeUnsatisfiable(format!(
                    "backend {name:?} cannot write into preallocated outputs (run_into)"
                )));
            }
            if !capabilities.zero_copy_inputs {
                return Err(InferenceError::RealtimeUnsatisfiable(format!(
                    "backend {name:?} cannot bind zero-copy inputs"
                )));
            }
        }

        let session = self.backend.load_model(self.source)?;
        let arena = TensorArena::new(self.policy);
        let mut engine =
            InferenceEngine::from_parts(session, capabilities, arena, name, self.device);

        for _ in 0..self.warmup {
            warmup_once(&mut engine)?;
        }

        Ok(engine)
    }
}

/// Run one inference on zero-filled inputs shaped from the model's input specs
/// (dynamic axes collapse to `1`), discarding the result.
fn warmup_once<S>(engine: &mut InferenceEngine<S>) -> InferenceResult<()>
where
    S: crate::backend::BackendSession,
{
    let tensors: Vec<Tensor> = engine
        .input_specs()
        .iter()
        .map(|spec| Tensor::zeros(spec.dtype, spec.shape.concrete_or_unit()))
        .collect();
    let views: Vec<_> = tensors.iter().map(|t| t.view()).collect();
    engine.run(&views)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clankers_tensor::{Shape, ShapeSpec, TensorView};

    use crate::backend::NoopBackend;

    #[test]
    fn builds_and_runs_a_noop_engine() {
        let mut engine = InferenceEngineBuilder::new(NoopBackend::identity(
            ShapeSpec::from_onnx_dims(&[1, 3]),
        ))
        .warmup(2)
        .build()
        .unwrap();

        assert_eq!(engine.backend_name(), "noop");
        assert_eq!(engine.input_specs().len(), 1);

        let data = vec![1.0f32, 2.0, 3.0];
        let shape = Shape::from([1, 3]);
        let out = engine
            .run(&[TensorView::from_f32(&data, &shape).unwrap()])
            .unwrap();
        assert_eq!(out[0].as_f32().unwrap(), data.as_slice());
    }

    #[test]
    fn rejects_unsupported_device() {
        // NoopBackend only advertises CPU.
        let err = InferenceEngineBuilder::new(NoopBackend::identity(ShapeSpec::from_onnx_dims(
            &[1, 3],
        )))
        .device(Device::Cuda(0))
        .build()
        .unwrap_err();
        assert!(matches!(err, InferenceError::UnsupportedDevice(_)));
    }

    fn model_config(device: &str, warmup: Option<u32>) -> clankers_core::config::ModelConfig {
        clankers_core::config::ModelConfig {
            source_framework: None,
            path: "m.onnx".into(),
            backend: "onnxruntime".into(),
            device: device.into(),
            warmup_runs: warmup,
            max_latency_ms: None,
            input: None,
            output: None,
        }
    }

    #[test]
    fn apply_model_config_sets_warmup_and_device() {
        let builder = InferenceEngineBuilder::new(NoopBackend::identity(
            ShapeSpec::from_onnx_dims(&[1, 4]),
        ))
        .apply_model_config(&model_config("cpu", Some(7)))
        .unwrap();
        assert_eq!(builder.warmup, 7);
        assert_eq!(builder.device, Device::Cpu);
    }

    #[test]
    fn apply_model_config_rejects_unparseable_device() {
        let result = InferenceEngineBuilder::new(NoopBackend::identity(
            ShapeSpec::from_onnx_dims(&[1, 4]),
        ))
        .apply_model_config(&model_config("tpu", None));
        assert!(matches!(
            result,
            Err(InferenceError::UnsupportedDevice(_))
        ));
    }

    #[test]
    fn warmup_does_not_leak_into_run_stats() {
        let mut engine = InferenceEngineBuilder::new(NoopBackend::identity(
            ShapeSpec::from_onnx_dims(&[1, 3]),
        ))
        .warmup(5)
        .build()
        .unwrap();
        let data = vec![0.0f32; 3];
        let shape = Shape::from([1, 3]);
        let (_out, stats) = engine
            .run_with_stats(&[TensorView::from_f32(&data, &shape).unwrap()])
            .unwrap();
        // Per-run deltas are independent of the warm-up runs already performed.
        assert_eq!(stats.clankers_copies, 0);
        assert_eq!(stats.allocations, 1); // one output placeholder
    }
}
