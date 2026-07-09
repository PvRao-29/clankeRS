//! An identity backend used to exercise the engine without a real runtime.
//!
//! `NoopBackend` copies each input straight to the corresponding output. It is
//! the backend behind every engine unit test: it needs no `.onnx` file, no
//! `ort` download, and its behaviour is trivially predictable.

use clankers_tensor::{DType, ShapeSpec};

use crate::backend::traits::{BackendRunStats, BackendSession, InferenceBackend};
use crate::backend::{BackendCapabilities, BackendTensor, TensorSpec};
use crate::inference::{InferenceError, ModelSource};

/// A backend whose model is the identity function: output `i` equals input `i`.
#[derive(Debug, Clone)]
pub struct NoopBackend {
    inputs: Vec<TensorSpec>,
    outputs: Vec<TensorSpec>,
}

impl NoopBackend {
    /// A single-input/single-output identity over `F32` tensors of `shape`.
    pub fn identity(shape: impl Into<ShapeSpec>) -> Self {
        let shape = shape.into();
        NoopBackend {
            inputs: vec![TensorSpec::new("input", DType::F32, shape.clone())],
            outputs: vec![TensorSpec::new("output", DType::F32, shape)],
        }
    }

    /// A general identity backend with explicit input/output specs. The two must
    /// have equal length (each input is copied to the matching output).
    pub fn new(inputs: Vec<TensorSpec>, outputs: Vec<TensorSpec>) -> Self {
        NoopBackend { inputs, outputs }
    }
}

impl InferenceBackend for NoopBackend {
    type Session = NoopSession;

    fn name(&self) -> &'static str {
        "noop"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            name: "noop",
            // The identity reads borrowed inputs directly and can fill a
            // caller-provided output buffer in place.
            zero_copy_inputs: true,
            supports_preallocated_outputs: true,
            supported_dtypes: vec![
                DType::Bool,
                DType::U8,
                DType::F16,
                DType::F32,
                DType::F64,
                DType::I32,
                DType::I64,
            ],
            supported_devices: vec![clankers_tensor::Device::Cpu],
        }
    }

    fn load_model(&self, _source: ModelSource) -> Result<NoopSession, InferenceError> {
        Ok(NoopSession {
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
        })
    }
}

/// A loaded [`NoopBackend`].
pub struct NoopSession {
    inputs: Vec<TensorSpec>,
    outputs: Vec<TensorSpec>,
}

impl BackendSession for NoopSession {
    fn input_specs(&self) -> &[TensorSpec] {
        &self.inputs
    }

    fn output_specs(&self) -> &[TensorSpec] {
        &self.outputs
    }

    fn run(
        &mut self,
        inputs: &[BackendTensor],
        outputs: &mut [BackendTensor],
    ) -> Result<BackendRunStats, InferenceError> {
        if inputs.len() != outputs.len() {
            return Err(InferenceError::Config(format!(
                "noop identity needs matching input/output counts, got {} in / {} out",
                inputs.len(),
                outputs.len()
            )));
        }

        let mut stats = BackendRunStats::ZERO;
        for (input, slot) in inputs.iter().zip(outputs.iter_mut()) {
            let src = input.view();
            // Identity: copy each input to its output. When the output is a
            // caller-preallocated buffer (`run_into`), fill it in place with no
            // allocation; otherwise produce a fresh owned tensor.
            if let Some(dst) = slot.bytes_mut() {
                dst.copy_from_slice(src.bytes());
                stats.record_copy(src.num_bytes());
            } else {
                let owned = input.to_owned_tensor();
                stats.record_copy(owned.num_bytes());
                *slot = BackendTensor::Owned(owned);
            }
        }
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clankers_tensor::{Shape, TensorView};

    #[test]
    fn identity_roundtrips_f32() {
        // Milestone 2 deliverable: NoopBackend round-trip on a [1,4] f32 tensor.
        let backend = NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4]));
        let mut session = backend.load_model(ModelSource::None).unwrap();

        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let shape = Shape::from([1, 4]);
        let view = TensorView::from_f32(&data, &shape).unwrap();

        let inputs = [BackendTensor::Borrowed(view)];
        let mut outputs = [BackendTensor::Owned(clankers_tensor::Tensor::zeros(
            DType::F32,
            Shape::from([1, 4]),
        ))];

        let stats = session.run(&inputs, &mut outputs).unwrap();
        assert_eq!(stats.backend_copies, 1);
        assert_eq!(outputs[0].view().as_f32().unwrap(), data.as_slice());
    }

    #[test]
    fn mismatched_counts_error() {
        let backend = NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4]));
        let mut session = backend.load_model(ModelSource::None).unwrap();
        let data = vec![0.0f32; 4];
        let shape = Shape::from([1, 4]);
        let view = TensorView::from_f32(&data, &shape).unwrap();
        let inputs = [BackendTensor::Borrowed(view)];
        let mut outputs: [BackendTensor; 0] = [];
        assert!(session.run(&inputs, &mut outputs).is_err());
    }
}
