//! Structured inference errors.

use clankers_tensor::TensorError;

/// An error raised while building or running an [`InferenceEngine`].
///
/// [`InferenceError::InvalidInput`] is the workhorse: it carries pre-rendered
/// `expected`/`got` descriptions so a mismatch reads like
/// `invalid input "image": expected F32 [1,3,224,224] NCHW, got U8 [480,640,3] HWC`.
///
/// [`InferenceEngine`]: crate::inference::InferenceEngine
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    /// A tensor view/owned construction failed (bad length or alignment).
    #[error("tensor error: {0}")]
    Tensor(#[from] TensorError),

    /// An input tensor did not match the backend's declared spec.
    #[error("invalid input {name:?}: expected {expected}, got {got}")]
    InvalidInput {
        name: String,
        expected: String,
        got: String,
    },

    /// The caller supplied the wrong number of inputs.
    #[error("wrong number of inputs: model expects {expected}, got {got}")]
    InputCount { expected: usize, got: usize },

    /// A named input did not correspond to any model input.
    #[error("unknown input {name:?}; model inputs are [{available}]")]
    UnknownInput { name: String, available: String },

    /// The caller supplied output buffers that don't match the model's outputs.
    #[error("invalid output {name:?}: expected {expected}, got {got}")]
    InvalidOutput {
        name: String,
        expected: String,
        got: String,
    },

    /// A device was requested that no backend can satisfy.
    #[error("unsupported device: {0}")]
    UnsupportedDevice(String),

    /// `strict_realtime` was requested but the backend cannot guarantee it.
    #[error("real-time guarantee not satisfiable: {0}")]
    RealtimeUnsatisfiable(String),

    /// The backend itself failed (e.g. an `ort` runtime error).
    #[error("backend {backend}: {message}")]
    Backend { backend: String, message: String },

    /// The model could not be loaded from its source.
    #[error("failed to load model from {origin}: {message}")]
    ModelLoad { origin: String, message: String },

    /// `run_into` was requested but the backend cannot write into caller buffers.
    #[error("backend {backend} does not support preallocated outputs")]
    UnsupportedPreallocatedOutputs { backend: String },

    /// Engine/backend was misconfigured at build time.
    #[error("configuration error: {0}")]
    Config(String),
}

impl InferenceError {
    /// Build a [`InferenceError::Backend`] from a named backend and any error.
    pub fn backend(name: impl Into<String>, message: impl std::fmt::Display) -> Self {
        InferenceError::Backend {
            backend: name.into(),
            message: message.to_string(),
        }
    }
}

/// Convenience alias for engine results.
pub type InferenceResult<T> = Result<T, InferenceError>;

impl From<InferenceError> for clankers_core::RobotError {
    fn from(e: InferenceError) -> Self {
        clankers_core::RobotError::Model(e.to_string())
    }
}

impl From<clankers_core::RobotError> for InferenceError {
    fn from(e: clankers_core::RobotError) -> Self {
        InferenceError::Config(e.to_string())
    }
}
