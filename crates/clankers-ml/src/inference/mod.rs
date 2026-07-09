//! The inference engine: the orchestration layer over a [`BackendSession`].
//!
//! [`InferenceEngine`] is the lower-level inference runtime used by [`Model`](crate::Model).
//! Most applications should use [`Model`] directly. Use `InferenceEngine` when
//! implementing custom backends, custom allocation policies, or advanced runtime
//! integrations.
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
pub use config::{
    engine_from_model_config, noop_engine_from_config, noop_engine_from_specs,
    onnx_engine_from_config, ConfiguredEngine,
};
pub use engine::InferenceEngine;
pub use error::{InferenceError, InferenceResult};
pub use model::ModelSource;
pub use named_outputs::NamedOutputs;
pub use profile::InferenceStats;
pub use runtime::ModelEngine;
