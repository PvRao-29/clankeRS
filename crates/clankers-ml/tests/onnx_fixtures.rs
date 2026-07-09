//! Integration tests for ONNX models under `tests/fixtures/onnx/`.
//!
//! Generate fixtures with `python3 scripts/make_onnx_fixtures.py`.

#![cfg(feature = "onnxruntime")]

use clankers_ml::backend::OnnxRuntimeBackend;
use clankers_ml::inference::InferenceEngine;
use clankers_ml::{Model, NamedOutputs};
use clankers_tensor::{DType, Layout, Shape, TensorView, TensorViewMut};

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/onnx")
        .join(name)
}

#[test]
fn onnx_multi_input_image_state_runs_named() {
    let mut model = Model::builder()
        .backend(OnnxRuntimeBackend::default())
        .load(fixture("policy_multi_input_image_state.onnx"))
        .unwrap();

    let image_bytes = vec![128u8; 1 * 64 * 64 * 3];
    let state = vec![0.1f32; 12];
    let image_shape = Shape::from([1, 64, 64, 3]);
    let state_shape = Shape::from([1, 12]);
    let image = TensorView::from_slice(
        &image_bytes,
        DType::U8,
        &image_shape,
        Layout::Contiguous,
    )
    .unwrap();
    let state_view = TensorView::from_f32(&state, &state_shape).unwrap();

    let outputs = model
        .run_named([("image", image), ("state", state_view)])
        .unwrap();
    assert!(outputs.contains("action"));
    assert_eq!(model.stats().unwrap().clankers_copies, 0);
}

#[test]
fn onnx_dynamic_output_shape_is_returned_correctly() {
    let mut engine = InferenceEngine::builder(OnnxRuntimeBackend::default())
        .model(fixture("dynamic_batch_identity.onnx"))
        .build()
        .unwrap();

    let shape = Shape::from([2, 3]);
    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let view = TensorView::from_f32(&data, &shape).unwrap();
    let outputs = engine.run(&[view]).unwrap();
    assert_eq!(outputs[0].shape().dims(), &[2, 3]);
}

#[test]
fn onnx_i64_input_binds_without_clankers_copy() {
    let mut model = Model::builder()
        .backend(OnnxRuntimeBackend::default())
        .load(fixture("identity_i64.onnx"))
        .unwrap();

    let data = vec![1i64, 2, 3, 4];
    let shape = Shape::from([1, 4]);
    let bytes = unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            std::mem::size_of_val(data.as_slice()),
        )
    };
    let view = TensorView::from_slice(bytes, DType::I64, &shape, Layout::Contiguous).unwrap();
    let name = model.engine().input_specs()[0].name.clone();
    let _ = model.run_named([(name.as_str(), view)]).unwrap();
    assert_eq!(model.stats().unwrap().clankers_copies, 0);
}

#[test]
fn onnx_u8_input_binds_without_clankers_copy() {
    let mut model = Model::builder()
        .backend(OnnxRuntimeBackend::default())
        .load(fixture("identity_u8.onnx"))
        .unwrap();

    let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let shape = Shape::from([1, 8]);
    let view = TensorView::from_slice(&data, DType::U8, &shape, Layout::Contiguous).unwrap();
    let name = model.engine().input_specs()[0].name.clone();
    let _ = model.run_named([(name.as_str(), view)]).unwrap();
    assert_eq!(model.stats().unwrap().clankers_copies, 0);
}

#[test]
fn onnx_run_into_rejects_preallocated_outputs() {
    let mut model = Model::builder()
        .backend(OnnxRuntimeBackend::default())
        .load(fixture("policy_single_f32.onnx"))
        .unwrap();

    let input = vec![1.0f32, 2.0, 3.0, 4.0];
    let in_shape = Shape::from([1, 4]);
    let view = TensorView::from_f32(&input, &in_shape).unwrap();
    let mut output_buf = [0.0f32; 2];
    let output = TensorViewMut::from_f32(&mut output_buf, Shape::from([1, 2])).unwrap();

    let err = model
        .run_into([("input", view)], &mut [output])
        .unwrap_err();
    assert!(err.to_string().contains("preallocated outputs"));
}

#[test]
fn onnx_two_inputs_one_output_does_not_assume_input_output_counts_match() {
    let mut model = Model::builder()
        .backend(OnnxRuntimeBackend::default())
        .load(fixture("two_inputs_one_output.onnx"))
        .unwrap();

    assert_eq!(model.engine().input_specs().len(), 2);
    assert_eq!(model.engine().output_specs().len(), 1);

    let a = vec![1.0f32; 4];
    let b = vec![2.0f32; 4];
    let shape = Shape::from([1, 4]);
    let outputs = model
        .run_named([
            ("a", TensorView::from_f32(&a, &shape).unwrap()),
            ("b", TensorView::from_f32(&b, &shape).unwrap()),
        ])
        .unwrap();
    assert!(outputs.contains("sum"));
}

#[test]
fn onnx_one_input_two_outputs_returns_named_outputs() {
    let mut model = Model::builder()
        .backend(OnnxRuntimeBackend::default())
        .load(fixture("one_input_two_outputs.onnx"))
        .unwrap();

    let input = vec![1.0f32, 2.0, 3.0];
    let in_shape = Shape::from([1, 3]);
    let view = TensorView::from_f32(&input, &in_shape).unwrap();
    let outputs: NamedOutputs = model.run_named([("input", view)]).unwrap();
    assert!(outputs.contains("out_a"));
    assert!(outputs.contains("out_b"));
}

#[test]
fn onnx_rejects_wrong_input_count_with_good_error() {
    let mut model = Model::builder()
        .backend(OnnxRuntimeBackend::default())
        .load(fixture("policy_single_f32.onnx"))
        .unwrap();

    let input = vec![0.0f32; 4];
    let in_shape = Shape::from([1, 4]);
    let view = TensorView::from_f32(&input, &in_shape).unwrap();
    let err = model.run_named([("not_a_real_input", view)]).unwrap_err();
    assert!(err.to_string().contains("unknown input"));
}

#[test]
fn golden_path_runs_named_zero_copy_inputs() {
    let mut model = Model::builder()
        .backend(OnnxRuntimeBackend::default())
        .load(fixture("policy_multi_input_image_state.onnx"))
        .unwrap();

    let image_bytes = vec![100u8; 1 * 64 * 64 * 3];
    let state = vec![0.0f32; 12];
    let image_shape = Shape::from([1, 64, 64, 3]);
    let state_shape = Shape::from([1, 12]);
    let image = TensorView::from_slice(
        &image_bytes,
        DType::U8,
        &image_shape,
        Layout::Contiguous,
    )
    .unwrap();
    let state_view = TensorView::from_f32(&state, &state_shape).unwrap();

    let outputs = model
        .run_named([("image", image), ("state", state_view)])
        .unwrap();
    assert!(outputs.contains("action"));
    assert_eq!(model.stats().unwrap().clankers_copies, 0);
}

#[test]
fn model_convenience_run_still_works_for_single_input() {
    let mut model = Model::load(fixture("policy_single_f32.onnx")).unwrap();
    let out = model.run(&[1.0, 2.0, 3.0, 4.0]).unwrap();
    assert_eq!(out.len(), 2);
}
