//! The ONNX Runtime backend, behind the modern [`InferenceBackend`] traits.
//!
//! This is the real backend the engine drives. It loads an `.onnx` model with
//! [`ort`], reflects each input/output into a [`TensorSpec`] (names, dtypes, and
//! shapes — with `-1` dims surfaced as [`Dim::Dynamic`](clankers_tensor::Dim)),
//! and binds contiguous inputs **zero-copy**: an f32/i64 [`TensorView`] is handed
//! to `ort` as a borrowing `TensorRef` with no intermediate `ndarray` allocation
//! and no `to_vec()`. Only the produced outputs are copied out of the runtime.
//!
//! The legacy [`onnx`](crate::backend::onnx) module (still used by the `Model`
//! compat wrapper) predates this and copies every input; it is left untouched.

use clankers_tensor::{DType, Device, Shape, ShapeSpec, Tensor, TensorView};
use ort::session::{Session, SessionInputValue};
use ort::value::{TensorElementType, TensorRef, ValueType};

use crate::backend::{
    BackendCapabilities, BackendRunStats, BackendSession, BackendTensor, InferenceBackend,
    TensorSpec,
};
use crate::inference::{InferenceError, InferenceResult, ModelSource};

/// The ONNX Runtime inference backend.
///
/// Cheap to construct and clone; the loaded model lives in the [`OnnxSession`]
/// this produces. Build an engine with
/// `InferenceEngine::builder(OnnxRuntimeBackend::default()).model(path).build()`.
#[derive(Debug, Clone, Default)]
pub struct OnnxRuntimeBackend {
    _private: (),
}

impl OnnxRuntimeBackend {
    /// A new backend with default (CPU) options.
    pub fn new() -> Self {
        OnnxRuntimeBackend::default()
    }
}

impl InferenceBackend for OnnxRuntimeBackend {
    type Session = OnnxSession;

    fn name(&self) -> &'static str {
        "onnxruntime"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            name: "onnxruntime",
            // Contiguous f32/i64 inputs are bound as borrowing `TensorRef`s.
            zero_copy_inputs: true,
            // ONNX Runtime writes outputs into caller buffers when `run_into` is used.
            supports_preallocated_outputs: false,
            supported_dtypes: vec![
                DType::F32,
                DType::F64,
                DType::F16,
                DType::U8,
                DType::I32,
                DType::I64,
                DType::Bool,
            ],
            supported_devices: vec![Device::Cpu],
        }
    }

    fn load_model(&self, source: ModelSource) -> InferenceResult<OnnxSession> {
        let mut builder =
            Session::builder().map_err(|e| InferenceError::backend("onnxruntime", e))?;

        let session = match source {
            ModelSource::Path(path) => {
                builder
                    .commit_from_file(&path)
                    .map_err(|e| InferenceError::ModelLoad {
                        origin: path.display().to_string(),
                        message: e.to_string(),
                    })?
            }
            ModelSource::Bytes(bytes) => {
                builder
                    .commit_from_memory(&bytes)
                    .map_err(|e| InferenceError::ModelLoad {
                        origin: format!("<{} in-memory bytes>", bytes.len()),
                        message: e.to_string(),
                    })?
            }
            ModelSource::None => {
                return Err(InferenceError::Config(
                    "onnxruntime backend requires a model path or bytes".into(),
                ));
            }
        };

        let inputs = specs_from_outlets(session.inputs())?;
        let outputs = specs_from_outlets(session.outputs())?;
        Ok(OnnxSession {
            session,
            inputs,
            outputs,
        })
    }
}

/// A loaded ONNX model.
pub struct OnnxSession {
    session: Session,
    inputs: Vec<TensorSpec>,
    outputs: Vec<TensorSpec>,
}

impl BackendSession for OnnxSession {
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
    ) -> InferenceResult<BackendRunStats> {
        if inputs.len() != self.inputs.len() {
            return Err(InferenceError::InputCount {
                expected: self.inputs.len(),
                got: inputs.len(),
            });
        }
        let mut stats = BackendRunStats::ZERO;

        // Materialise the views once so they outlive the `run` call; the ort
        // inputs below borrow their data.
        let views: Vec<TensorView> = inputs.iter().map(|bt| bt.view()).collect();
        let mut session_inputs: Vec<(String, SessionInputValue)> = Vec::with_capacity(views.len());
        for (view, spec) in views.iter().zip(self.inputs.iter()) {
            session_inputs.push((spec.name.clone(), input_value(view, &spec.name)?));
        }

        let session_outputs = self
            .session
            .run(session_inputs)
            .map_err(|e| InferenceError::backend("onnxruntime", e))?;

        for (slot, spec) in outputs.iter_mut().zip(self.outputs.iter()) {
            let value = &session_outputs[spec.name.as_str()];
            let tensor = extract_output(value, spec)?;
            stats.record_copy(tensor.num_bytes());
            *slot = BackendTensor::Owned(tensor);
        }

        Ok(stats)
    }
}

/// Reflect a model's input/output metadata into [`TensorSpec`]s.
fn specs_from_outlets(outlets: &[ort::value::Outlet]) -> InferenceResult<Vec<TensorSpec>> {
    outlets
        .iter()
        .map(|outlet| match outlet.dtype() {
            ValueType::Tensor { ty, shape, .. } => {
                let dtype = map_element_type(*ty).ok_or_else(|| {
                    InferenceError::backend(
                        "onnxruntime",
                        format!("tensor {:?} has unsupported element type {ty:?}", outlet.name()),
                    )
                })?;
                Ok(TensorSpec::new(
                    outlet.name().to_string(),
                    dtype,
                    ShapeSpec::from_onnx_dims(&shape[..]),
                ))
            }
            other => Err(InferenceError::backend(
                "onnxruntime",
                format!("{:?} is not a tensor ({other:?})", outlet.name()),
            )),
        })
        .collect()
}

/// Build a borrowing `ort` input value from a caller view — the zero-copy path.
fn input_value<'v>(
    view: &TensorView<'v>,
    name: &str,
) -> InferenceResult<SessionInputValue<'v>> {
    let shape: Vec<i64> = view.shape().dims().iter().map(|&d| d as i64).collect();
    let value = match view.dtype() {
        DType::F32 => {
            let data: &'v [f32] = view.as_f32()?;
            let tensor = TensorRef::from_array_view((shape, data))
                .map_err(|e| InferenceError::backend("onnxruntime", format!("input {name:?}: {e}")))?;
            SessionInputValue::from(tensor)
        }
        DType::I64 => {
            let data: &'v [i64] = view.as_i64()?;
            let tensor = TensorRef::from_array_view((shape, data))
                .map_err(|e| InferenceError::backend("onnxruntime", format!("input {name:?}: {e}")))?;
            SessionInputValue::from(tensor)
        }
        DType::U8 => {
            let data: &'v [u8] = view.as_u8()?;
            let tensor = TensorRef::from_array_view((shape, data))
                .map_err(|e| InferenceError::backend("onnxruntime", format!("input {name:?}: {e}")))?;
            SessionInputValue::from(tensor)
        }
        other => {
            return Err(InferenceError::backend(
                "onnxruntime",
                format!("input {name:?}: dtype {other} is not supported for zero-copy binding"),
            ));
        }
    };
    Ok(value)
}

/// Copy a produced output value out of the runtime into an owned [`Tensor`].
fn extract_output(value: &ort::value::DynValue, spec: &TensorSpec) -> InferenceResult<Tensor> {
    let name = &spec.name;
    match spec.dtype {
        DType::F32 => {
            let (shape, data) = value.try_extract_tensor::<f32>().map_err(|e| {
                InferenceError::backend("onnxruntime", format!("output {name:?}: {e}"))
            })?;
            Tensor::from_f32_vec(to_shape(shape), data.to_vec()).map_err(InferenceError::from)
        }
        DType::I64 => {
            let (shape, data) = value.try_extract_tensor::<i64>().map_err(|e| {
                InferenceError::backend("onnxruntime", format!("output {name:?}: {e}"))
            })?;
            // SAFETY: `data` is a valid `&[i64]`; reinterpret as its byte image.
            let bytes = unsafe {
                std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
            };
            Tensor::from_bytes(DType::I64, to_shape(shape), bytes).map_err(InferenceError::from)
        }
        DType::U8 => {
            let (shape, data) = value.try_extract_tensor::<u8>().map_err(|e| {
                InferenceError::backend("onnxruntime", format!("output {name:?}: {e}"))
            })?;
            Tensor::from_bytes(DType::U8, to_shape(shape), data).map_err(InferenceError::from)
        }
        other => Err(InferenceError::backend(
            "onnxruntime",
            format!("output {name:?}: dtype {other} extraction is not supported"),
        )),
    }
}

/// Convert an `ort` concrete output shape into a [`Shape`].
fn to_shape(shape: &ort::value::Shape) -> Shape {
    Shape::new(shape.iter().map(|&d| d.max(0) as usize))
}

/// Map an `ort` element type to our [`DType`], or `None` if unsupported.
fn map_element_type(ty: TensorElementType) -> Option<DType> {
    use TensorElementType as T;
    Some(match ty {
        T::Float32 => DType::F32,
        T::Float64 => DType::F64,
        T::Float16 => DType::F16,
        T::Uint8 => DType::U8,
        T::Int32 => DType::I32,
        T::Int64 => DType::I64,
        T::Bool => DType::Bool,
        _ => return None,
    })
}
