//! Integration tests for the modular [`InferenceEngine`] on the ONNX Runtime
//! backend. These need the `onnxruntime` feature (on by default) and the sample
//! models under `sample_data/models/`.

#![cfg(feature = "onnxruntime")]

use clankers_ml::inference::InferenceEngine;
use clankers_ml::OnnxRuntimeBackend;
use clankers_tensor::{DType, Tensor, TensorView};

fn sample_model(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../sample_data/models")
        .join(name)
}

#[test]
fn loads_specs_from_onnx_metadata() {
    let engine = InferenceEngine::builder(OnnxRuntimeBackend::default())
        .model(sample_model("detector.onnx"))
        .build()
        .unwrap();

    assert_eq!(engine.backend_name(), "onnxruntime");
    assert!(!engine.input_specs().is_empty(), "inputs reflected from model");
    assert!(!engine.output_specs().is_empty(), "outputs reflected from model");
    // Every input carries a real name from the ONNX graph.
    assert!(engine.input_specs().iter().all(|s| !s.name.is_empty()));
}

#[test]
fn zero_copy_input_runs_without_conversion_copies() {
    let mut engine = InferenceEngine::builder(OnnxRuntimeBackend::default())
        .model(sample_model("detector.onnx"))
        .warmup(1)
        .build()
        .unwrap();

    let spec = &engine.input_specs()[0];
    assert_eq!(spec.dtype, DType::F32, "sample detector takes an f32 input");
    let shape = spec.shape.concrete_or_unit();
    let input = Tensor::zeros(DType::F32, shape.clone());
    let view = TensorView::from_f32(input.as_f32().unwrap(), &shape).unwrap();

    let (outputs, stats) = engine.run_with_stats(&[view]).unwrap();
    assert!(!outputs.is_empty(), "detector should produce output tensors");
    assert!(
        outputs[0].num_elements() > 0,
        "output tensor should be non-empty"
    );
    // The engine bound the contiguous f32 view directly — no conversion copy.
    assert_eq!(stats.clankers_copies, 0, "matching f32 input must be zero-copy");
    assert!(stats.is_zero_copy());
}

#[test]
fn rejects_wrong_input_dtype_with_structured_error() {
    let mut engine = InferenceEngine::builder(OnnxRuntimeBackend::default())
        .model(sample_model("detector.onnx"))
        .build()
        .unwrap();

    let shape = engine.input_specs()[0].shape.concrete_or_unit();
    // Feed a U8 buffer where the model expects F32.
    let bytes = vec![0u8; shape.num_elements()];
    let view =
        TensorView::from_slice(&bytes, DType::U8, &shape, clankers_tensor::Layout::Contiguous)
            .unwrap();
    let err = engine.run(&[view]).unwrap_err();
    assert!(
        matches!(err, clankers_ml::InferenceError::InvalidInput { .. }),
        "dtype mismatch should be a structured InvalidInput error, got {err:?}"
    );
}

#[test]
fn model_from_memory_bytes_loads() {
    let bytes = std::fs::read(sample_model("policy.onnx")).unwrap();
    let engine = InferenceEngine::builder(OnnxRuntimeBackend::default())
        .model(bytes)
        .build()
        .unwrap();
    assert!(!engine.input_specs().is_empty());
}
