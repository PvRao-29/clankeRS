//! The inference engine: orchestration over a [`BackendSession`](crate::backend::BackendSession).
//!
//! [`InferenceEngine`] is the lower-level inference runtime used by [`Model`](crate::Model).
//! Most applications should use [`Model`](crate::Model) directly. Use `InferenceEngine` when
//! implementing custom backends, custom allocation policies, or advanced runtime integrations.
//!
//! ## Typical power-user flow
//!
//! ```no_run
//! # fn example() -> clankers_ml::InferenceResult<()> {
//! use clankers_ml::backend::NoopBackend;
//! use clankers_ml::inference::InferenceEngine;
//! use clankers_tensor::{ShapeSpec, TensorView};
//!
//! # #[cfg(feature = "onnxruntime")]
//! # {
//! use clankers_ml::backend::OnnxRuntimeBackend;
//! let mut engine = InferenceEngine::builder(OnnxRuntimeBackend::default())
//!     .model("models/policy.onnx")
//!     .build()?;
//! let shape = engine.input_specs()[0].shape.concrete_or_unit();
//! let input = vec![0.0f32; shape.num_elements()];
//! let view = TensorView::from_f32(&input, &shape)?;
//! let (_outputs, stats) = engine.run_with_stats(&[view])?;
//! let _ = stats;
//! # }
//!
//! // Sim-only build (no ONNX):
//! let mut noop = InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
//!     .build()?;
//! let _ = &mut noop;
//! # Ok(())
//! # }
//! ```
//!
//! [`BackendSession`]: crate::backend::BackendSession

mod builder;
mod config;
mod engine;
mod error;
mod model;
mod named_outputs;
mod profile;
mod runtime;
mod session;

pub use builder::InferenceEngineBuilder;
#[cfg(feature = "onnxruntime")]
pub use config::onnx_engine_from_config;
pub use config::{
    engine_from_model_config, noop_engine_from_config, noop_engine_from_specs, ConfiguredEngine,
};
pub use engine::InferenceEngine;
pub use error::{InferenceError, InferenceResult};
pub use model::ModelSource;
pub use named_outputs::NamedOutputs;
pub use profile::InferenceStats;
pub use runtime::ModelEngine;
