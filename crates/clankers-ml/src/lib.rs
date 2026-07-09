//! ML inference and model deployment for clankeRS.
//!
//! [`Model`] is the main optimized inference API: load a model, pass zero-copy
//! [`TensorView`](clankers_tensor::TensorView) inputs, and read named outputs
//! with per-run profiling. Under the hood it uses the modular
//! [`InferenceEngine`](inference::InferenceEngine) and pluggable backends
//! ([`OnnxRuntimeBackend`](backend::OnnxRuntimeBackend), [`NoopBackend`](backend::NoopBackend), …).

pub mod backend;
pub mod inference;
pub mod model;
pub mod validation;

#[allow(deprecated)]
pub use backend::ModelBackend as LegacyModelBackend;
pub use backend::{
    BackendCapabilities, BackendRunStats, BackendSession, BackendTensor, InferenceBackend,
    NoopBackend, TensorSpec,
};
#[cfg(feature = "onnxruntime")]
pub use backend::{OnnxRuntimeBackend, OnnxSession};
pub use inference::{
    engine_from_model_config, noop_engine_from_config, noop_engine_from_specs,
    onnx_engine_from_config, ConfiguredEngine, InferenceEngine, InferenceEngineBuilder,
    InferenceError, InferenceResult, InferenceStats, ModelEngine, ModelSource, NamedOutputs,
};
pub use model::{Model, ModelBuilder, ModelMetadata, RuntimeBackend};
pub use validation::{ModelValidator, ValidationReport};
