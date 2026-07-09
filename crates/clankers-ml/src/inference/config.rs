//! Bridge [`ModelConfig`](clankers_core::config::ModelConfig) into engine builders.
//!
//! `clankers-core` stays backend-agnostic; this module maps TOML model settings
//! onto the concrete [`InferenceEngineBuilder`] for each supported backend.

use clankers_core::config::{ModelBackendKind, ModelConfig};

use crate::backend::{NoopBackend, NoopSession, TensorSpec};
use crate::inference::engine::InferenceEngine;
use crate::inference::{InferenceError, InferenceResult, ModelSource};

#[cfg(feature = "onnxruntime")]
use crate::backend::{OnnxRuntimeBackend, OnnxSession};

/// A loaded engine plus its backend kind, for callers that select the backend
/// from config at runtime.
pub enum ConfiguredEngine {
    #[cfg(feature = "onnxruntime")]
    OnnxRuntime(InferenceEngine<OnnxSession>),
    Noop(InferenceEngine<NoopSession>),
}

impl ConfiguredEngine {
    /// The backend name (`"onnxruntime"`, `"noop"`, …).
    pub fn backend_name(&self) -> &str {
        match self {
            #[cfg(feature = "onnxruntime")]
            Self::OnnxRuntime(engine) => engine.backend_name(),
            Self::Noop(engine) => engine.backend_name(),
        }
    }
}

/// Build an [`InferenceEngine`] from a [`ModelConfig`] and model source.
///
/// Maps `backend`, `warmup_runs`, and `device` from the config. For the `noop`
/// backend, `model` is only used to reflect ONNX input/output specs when the
/// `onnxruntime` feature is enabled; otherwise the caller must supply specs via
/// [`noop_engine_from_specs`].
pub fn engine_from_model_config(
    config: &ModelConfig,
    model: impl Into<ModelSource>,
) -> InferenceResult<ConfiguredEngine> {
    match config.backend_kind().map_err(InferenceError::from)? {
        ModelBackendKind::OnnxRuntime => {
            #[cfg(feature = "onnxruntime")]
            {
                Ok(ConfiguredEngine::OnnxRuntime(onnx_engine_from_config(config, model)?))
            }
            #[cfg(not(feature = "onnxruntime"))]
            {
                let _ = (config, model);
                Err(InferenceError::Config(
                    "onnxruntime backend requires the onnxruntime feature".into(),
                ))
            }
        }
        ModelBackendKind::Noop => Ok(ConfiguredEngine::Noop(noop_engine_from_config(
            config, model,
        )?)),
    }
}

/// Build an ONNX Runtime engine with config-driven warmup and device settings.
#[cfg(feature = "onnxruntime")]
pub fn onnx_engine_from_config(
    config: &ModelConfig,
    model: impl Into<ModelSource>,
) -> InferenceResult<InferenceEngine<OnnxSession>> {
    InferenceEngine::builder(OnnxRuntimeBackend::default())
        .model(model)
        .apply_model_config(config)?
        .build()
}

/// Build a `NoopBackend` engine that mirrors a real model's tensor specs.
///
/// When the `onnxruntime` feature is enabled, `model` is loaded once to read
/// input/output metadata, then discarded. The noop identity requires equal input
/// and output counts; models that violate this return a configuration error.
pub fn noop_engine_from_config(
    config: &ModelConfig,
    model: impl Into<ModelSource>,
) -> InferenceResult<InferenceEngine<NoopSession>> {
    let model = model.into();
    #[cfg(feature = "onnxruntime")]
    let (inputs, outputs) = {
        let probe = InferenceEngine::builder(OnnxRuntimeBackend::default())
            .model(model)
            .build()?;
        let inputs = probe.input_specs().to_vec();
        let outputs = probe.output_specs().to_vec();
        if inputs.len() != outputs.len() {
            return Err(InferenceError::Config(format!(
                "noop backend requires equal input/output counts, model has {} in / {} out",
                inputs.len(),
                outputs.len()
            )));
        }
        (inputs, outputs)
    };
    #[cfg(not(feature = "onnxruntime"))]
    let (inputs, outputs) = {
        let _ = model;
        return Err(InferenceError::Config(
            "noop backend with model-derived specs requires the onnxruntime feature".into(),
        ));
    };

    InferenceEngine::builder(NoopBackend::new(inputs, outputs))
        .apply_model_config(config)?
        .build()
}

/// Build a noop engine from explicit tensor specs (no ONNX probe).
pub fn noop_engine_from_specs(
    config: &ModelConfig,
    inputs: Vec<TensorSpec>,
    outputs: Vec<TensorSpec>,
) -> InferenceResult<InferenceEngine<NoopSession>> {
    if inputs.len() != outputs.len() {
        return Err(InferenceError::Config(format!(
            "noop backend requires equal input/output counts, got {} in / {} out",
            inputs.len(),
            outputs.len()
        )));
    }
    InferenceEngine::builder(NoopBackend::new(inputs, outputs))
        .apply_model_config(config)?
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clankers_tensor::ShapeSpec;

    fn sample_config() -> ModelConfig {
        ModelConfig {
            source_framework: None,
            path: "detector.onnx".into(),
            backend: "onnxruntime".into(),
            device: "cpu".into(),
            warmup_runs: Some(1),
            max_latency_ms: None,
            input: None,
            output: None,
        }
    }

    fn sample_model() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../sample_data/models/detector.onnx")
    }

    #[test]
    #[cfg(feature = "onnxruntime")]
    fn onnx_engine_from_config_loads_sample_model() {
        let engine = onnx_engine_from_config(&sample_config(), sample_model()).unwrap();
        assert_eq!(engine.backend_name(), "onnxruntime");
        assert!(!engine.input_specs().is_empty());
    }

    #[test]
    #[cfg(feature = "onnxruntime")]
    fn engine_from_model_config_selects_onnx() {
        let engine = engine_from_model_config(&sample_config(), sample_model()).unwrap();
        assert_eq!(engine.backend_name(), "onnxruntime");
    }

    #[test]
    #[cfg(feature = "onnxruntime")]
    fn noop_engine_from_config_mirrors_specs() {
        let mut config = sample_config();
        config.backend = "noop".into();
        let engine = noop_engine_from_config(&config, sample_model()).unwrap();
        assert_eq!(engine.backend_name(), "noop");
        assert_eq!(
            engine.input_specs().len(),
            engine.output_specs().len(),
            "noop mirrors a model with matched I/O counts"
        );
    }

    #[test]
    fn noop_engine_from_specs_rejects_mismatched_counts() {
        let config = ModelConfig {
            source_framework: None,
            path: "m.onnx".into(),
            backend: "noop".into(),
            device: "cpu".into(),
            warmup_runs: None,
            max_latency_ms: None,
            input: None,
            output: None,
        };
        let spec = |n| {
            crate::backend::TensorSpec::new(
                n,
                clankers_tensor::DType::F32,
                ShapeSpec::from_onnx_dims(&[1, 4]),
            )
        };
        let err = noop_engine_from_specs(&config, vec![spec("in")], vec![spec("a"), spec("b")])
            .unwrap_err();
        assert!(matches!(err, InferenceError::Config(_)));
    }
}
