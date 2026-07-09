//! # ML inference and model deployment
//!
//! Load ONNX models, run optimized inference with zero-copy inputs, and validate Rust
//! output against offline PyTorch references.
//!
//! ## Start here: [`Model`]
//!
//! [`Model`] is the main API. It wraps a modular [`InferenceEngine`] and a pluggable
//! backend ([`OnnxRuntimeBackend`], [`NoopBackend`], …).
//!
//! ```no_run
//! # #[cfg(feature = "onnxruntime")]
//! # fn example() -> clankers_ml::InferenceResult<()> {
//! use clankers_ml::backend::OnnxRuntimeBackend;
//! use clankers_ml::Model;
//! use clankers_tensor::{DType, Layout, Shape, TensorView};
//!
//! let mut model = Model::builder()
//!     .backend(OnnxRuntimeBackend::default())
//!     .load("models/policy.onnx")?;
//!
//! let shape = Shape::from([1, 4]);
//! let view = TensorView::from_f32(&[1.0, 2.0, 3.0, 4.0], &shape)?;
//! let outputs = model.run_named([("input", view)])?;
//!
//! if let Some(stats) = model.stats() {
//!     assert_eq!(stats.clankers_copies, 0, "matching f32 input should not be copied by clankeRS");
//! }
//! let _ = outputs;
//! # Ok(())
//! # }
//! ```
//!
//! Sim-only builds (no ONNX) use [`NoopBackend`] the same way:
//!
//! ```no_run
//! # #[cfg(not(feature = "onnxruntime"))]
//! # fn example() -> clankers_ml::InferenceResult<()> {
//! use clankers_ml::backend::NoopBackend;
//! use clankers_ml::Model;
//! use clankers_tensor::{Shape, ShapeSpec, TensorView};
//!
//! let mut model = Model::builder()
//!     .backend(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
//!     .build()?;
//! let shape = Shape::from([1, 4]);
//! let view = TensorView::from_f32(&[1.0, 2.0, 3.0, 4.0], &shape)?;
//! let _outputs = model.run_named([("input", view)])?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Power users: [`InferenceEngine`]
//!
//! Use [`InferenceEngine`] directly when implementing custom [`InferenceBackend`] traits,
//! custom allocation policies ([`clankers_tensor::AllocationPolicy`]), or
//! [`InferenceEngine::run_into`] with preallocated output buffers.
//!
//! ## Validation
//!
//! [`ModelValidator`] compares ONNX runtime output to stored PyTorch `expected_output.json`
//! files — no PyTorch required at validation time.
//!
//! ## Legacy API
//!
//! [`LegacyModelBackend`] is the old flat `run(&[f32])` trait (deprecated since **0.1.2**).
//! New code should use [`Model`] or [`InferenceEngine`].

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
#[cfg(feature = "onnxruntime")]
pub use inference::onnx_engine_from_config;
pub use inference::{
    engine_from_model_config, noop_engine_from_config, noop_engine_from_specs, ConfiguredEngine,
    InferenceEngine, InferenceEngineBuilder, InferenceError, InferenceResult, InferenceStats,
    ModelEngine, ModelSource, NamedOutputs,
};
pub use model::{Model, ModelBuilder, ModelMetadata, RuntimeBackend};
pub use validation::{ModelValidator, ValidationReport};
