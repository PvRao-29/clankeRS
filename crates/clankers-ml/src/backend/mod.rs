//! Inference backends.
//!
//! The modern abstraction is a factory/session split: an [`InferenceBackend`]
//! loads a model into a [`BackendSession`] which the [`InferenceEngine`] drives.
//! [`NoopBackend`] is an identity implementation used throughout the tests; the
//! ONNX Runtime backend (added in a later milestone) lives in an `onnxruntime`
//! submodule behind the `onnxruntime` feature.
//!
//! The original flat-buffer [`ModelBackend`] trait is retained as a deprecated
//! low-level shim. New code should use [`Model`](crate::Model) or
//! [`InferenceEngine`](crate::inference::InferenceEngine).
//!
//! [`InferenceEngine`]: crate::inference::InferenceEngine

mod capability;
pub mod noop;
#[cfg(feature = "onnxruntime")]
pub mod onnxruntime;
mod spec;
mod tensor;
mod traits;

pub use capability::BackendCapabilities;
pub use noop::{NoopBackend, NoopSession};
#[cfg(feature = "onnxruntime")]
pub use onnxruntime::{OnnxRuntimeBackend, OnnxSession};
pub use spec::{describe_view, TensorSpec};
pub use tensor::BackendTensor;
pub use traits::{BackendRunStats, BackendSession, InferenceBackend};

// ---------------------------------------------------------------------------
// Legacy flat-buffer backend (deprecated).
//
// This is the original `run(&[f32])` abstraction predating the modular engine.
// New code should use [`Model`](crate::Model) or [`InferenceEngine`](crate::inference::InferenceEngine).
// ---------------------------------------------------------------------------

use std::time::{Duration, Instant};

use clankers_core::{RobotError, RobotResult};

/// Legacy flat-buffer inference backend.
#[deprecated(
    since = "0.1.2",
    note = "use `clankers_ml::Model` or `clankers_ml::inference::InferenceEngine` with an `InferenceBackend` instead"
)]
pub trait ModelBackend: Send + Sync {
    fn name(&self) -> &str;
    fn input_shapes(&self) -> Vec<Vec<usize>>;
    fn output_shapes(&self) -> Vec<Vec<usize>>;
    fn run(&self, input: &[f32]) -> RobotResult<Vec<f32>>;
    fn run_with_latency(&self, input: &[f32]) -> RobotResult<(Vec<f32>, Duration)> {
        let start = Instant::now();
        let output = self.run(input)?;
        Ok((output, start.elapsed()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    OnnxRuntime,
}

impl BackendKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OnnxRuntime => "onnxruntime",
        }
    }
}

#[cfg(feature = "onnxruntime")]
pub mod onnx {
    //! Legacy single-tensor ONNX wrapper used by the `Model` compat path.
    //!
    //! The engine-native ONNX backend is [`super::onnxruntime`]; this module is
    //! the thin `run(&[f32])` shim that predates it.
    use std::path::Path;
    use std::sync::Mutex;

    use ndarray::ArrayD;
    use ort::session::Session;
    use ort::value::{Tensor, ValueType};

    #[allow(deprecated)]
    use super::ModelBackend;
    use super::{RobotError, RobotResult};

    pub struct OnnxBackend {
        session: Mutex<Session>,
        input_shape: Vec<usize>,
        output_shape: Vec<usize>,
    }

    fn shape_from_outlet(outlet: &ort::value::Outlet) -> Vec<usize> {
        match outlet.dtype() {
            ValueType::Tensor { shape, .. } => shape
                .iter()
                .map(|&d| if d < 0 { 1 } else { d as usize })
                .collect(),
            _ => vec![1, 10],
        }
    }

    impl OnnxBackend {
        pub fn load(path: impl AsRef<Path>) -> RobotResult<Self> {
            let path = path.as_ref();
            let mut builder = Session::builder().map_err(|e| RobotError::Model(e.to_string()))?;
            let session = builder
                .commit_from_file(path)
                .map_err(|e| RobotError::Model(format!("load onnx '{}': {e}", path.display())))?;

            let inputs = session.inputs();
            let outputs = session.outputs();
            if inputs.is_empty() || outputs.is_empty() {
                return Err(RobotError::Model(
                    "onnx model has no inputs or outputs".into(),
                ));
            }

            let input_shape = shape_from_outlet(&inputs[0]);
            let output_shape = shape_from_outlet(&outputs[0]);

            Ok(Self {
                session: Mutex::new(session),
                input_shape,
                output_shape,
            })
        }
    }

    #[allow(deprecated)]
    impl ModelBackend for OnnxBackend {
        fn name(&self) -> &str {
            "onnxruntime"
        }

        fn input_shapes(&self) -> Vec<Vec<usize>> {
            vec![self.input_shape.clone()]
        }

        fn output_shapes(&self) -> Vec<Vec<usize>> {
            vec![self.output_shape.clone()]
        }

        fn run(&self, input: &[f32]) -> RobotResult<Vec<f32>> {
            let expected: usize = self.input_shape.iter().product();
            if input.len() != expected {
                return Err(RobotError::Model(format!(
                    "input size mismatch: expected {expected}, got {}",
                    input.len()
                )));
            }

            let array = ArrayD::from_shape_vec(self.input_shape.clone(), input.to_vec())
                .map_err(|e| RobotError::Model(e.to_string()))?;

            let input_tensor =
                Tensor::from_array(array).map_err(|e| RobotError::Model(e.to_string()))?;

            let mut session = self
                .session
                .lock()
                .map_err(|e| RobotError::Model(e.to_string()))?;

            let outputs = session
                .run(ort::inputs![input_tensor])
                .map_err(|e| RobotError::Model(e.to_string()))?;

            let output = &outputs[0];

            let (_shape, data) = output
                .try_extract_tensor::<f32>()
                .map_err(|e| RobotError::Model(e.to_string()))?;

            Ok(data.to_vec())
        }
    }
}

#[allow(deprecated)]
pub fn load_onnx(path: impl AsRef<std::path::Path>) -> RobotResult<Box<dyn ModelBackend>> {
    #[cfg(feature = "onnxruntime")]
    {
        Ok(Box::new(onnx::OnnxBackend::load(path)?))
    }
    #[cfg(not(feature = "onnxruntime"))]
    {
        let _ = path;
        Err(RobotError::Model("onnxruntime feature not enabled".into()))
    }
}
