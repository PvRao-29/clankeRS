//! Integration tests calling the C API from Rust.

use std::ffi::CString;
use std::ptr;

use clankers_ffi::{
    clankers_abi_version, clankers_last_error_code, clankers_last_error_message, clankers_version,
    ClankersDType, ClankersEngine, ClankersInferenceStats, ClankersLayout, ClankersShape,
    ClankersStatus, ClankersTensor, ClankersTensorView, ClankersTensorViewMut,
    CLANKERS_ABI_VERSION,
};

extern "C" {
    fn clankers_engine_builder_new() -> *mut clankers_ffi::ClankersEngineBuilder;
    fn clankers_engine_builder_set_backend(
        builder: *mut clankers_ffi::ClankersEngineBuilder,
        backend: *const i8,
    ) -> ClankersStatus;
    fn clankers_engine_builder_set_model_path(
        builder: *mut clankers_ffi::ClankersEngineBuilder,
        path: *const i8,
    ) -> ClankersStatus;
    fn clankers_engine_builder_set_noop_shape(
        builder: *mut clankers_ffi::ClankersEngineBuilder,
        dims: *const usize,
        rank: usize,
    ) -> ClankersStatus;
    fn clankers_engine_builder_set_strict_realtime(
        builder: *mut clankers_ffi::ClankersEngineBuilder,
        enabled: bool,
    ) -> ClankersStatus;
    fn clankers_engine_builder_build(
        builder: *mut clankers_ffi::ClankersEngineBuilder,
        out_engine: *mut *mut ClankersEngine,
    ) -> ClankersStatus;
    fn clankers_engine_destroy(engine: *mut ClankersEngine) -> ClankersStatus;
    fn clankers_engine_input_count(engine: *const ClankersEngine) -> usize;
    fn clankers_engine_run(
        engine: *mut ClankersEngine,
        inputs: *const ClankersTensorView,
        input_count: usize,
        out_outputs: *mut *mut *mut ClankersTensor,
        out_output_count: *mut usize,
    ) -> ClankersStatus;
    fn clankers_engine_run_with_stats(
        engine: *mut ClankersEngine,
        inputs: *const ClankersTensorView,
        input_count: usize,
        out_outputs: *mut *mut *mut ClankersTensor,
        out_output_count: *mut usize,
        out_stats: *mut ClankersInferenceStats,
    ) -> ClankersStatus;
    fn clankers_engine_run_into(
        engine: *mut ClankersEngine,
        inputs: *const ClankersTensorView,
        input_count: usize,
        outputs: *mut ClankersTensorViewMut,
        output_count: usize,
        out_stats: *mut ClankersInferenceStats,
    ) -> ClankersStatus;
    fn clankers_tensor_view_from_external(
        data: *const u8,
        byte_len: usize,
        dtype: ClankersDType,
        shape: ClankersShape,
        layout: ClankersLayout,
        out_view: *mut ClankersTensorView,
    ) -> ClankersStatus;
    fn clankers_tensor_view_mut_from_external(
        data: *mut u8,
        byte_len: usize,
        dtype: ClankersDType,
        shape: ClankersShape,
        layout: ClankersLayout,
        out_view: *mut ClankersTensorViewMut,
    ) -> ClankersStatus;
    fn clankers_tensor_data(tensor: *const ClankersTensor) -> *const u8;
    fn clankers_tensor_byte_len(tensor: *const ClankersTensor) -> usize;
    fn clankers_tensor_destroy(tensor: *mut ClankersTensor) -> ClankersStatus;
    fn clankers_output_array_destroy(outputs: *mut *mut ClankersTensor, count: usize);
}

fn cstr(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn noop_engine(shape: [usize; 2]) -> *mut ClankersEngine {
    unsafe {
        let builder = clankers_engine_builder_new();
        assert!(!builder.is_null());
        assert_eq!(
            clankers_engine_builder_set_backend(builder, cstr("noop").as_ptr()),
            ClankersStatus::Ok
        );
        assert_eq!(
            clankers_engine_builder_set_noop_shape(builder, shape.as_ptr(), shape.len()),
            ClankersStatus::Ok
        );
        let mut engine = ptr::null_mut();
        assert_eq!(
            clankers_engine_builder_build(builder, &mut engine),
            ClankersStatus::Ok
        );
        assert!(!engine.is_null());
        engine
    }
}

fn f32_view(data: &[f32], shape: ClankersShape) -> ClankersTensorView {
    let mut view = ClankersTensorView {
        data: ptr::null(),
        byte_len: 0,
        dtype: ClankersDType::F32,
        shape,
        layout: ClankersLayout::Contiguous,
        device: clankers_ffi::ClankersDevice::Cpu,
    };
    let status = unsafe {
        clankers_tensor_view_from_external(
            data.as_ptr() as *const u8,
            std::mem::size_of_val(data),
            ClankersDType::F32,
            shape,
            ClankersLayout::Contiguous,
            &mut view,
        )
    };
    assert_eq!(status, ClankersStatus::Ok);
    view
}

fn shape_1x4() -> ClankersShape {
    ClankersShape {
        dims: [1, 4, 0, 0, 0, 0],
        rank: 2,
    }
}

#[test]
fn version_and_abi() {
    unsafe {
        let ver = std::ffi::CStr::from_ptr(clankers_version());
        assert!(ver.to_str().unwrap().starts_with("0.1."));
        assert_eq!(clankers_abi_version(), CLANKERS_ABI_VERSION);
    }
}

#[test]
fn null_engine_run_returns_error() {
    let mut outputs: *mut *mut ClankersTensor = ptr::null_mut();
    let mut count = 0usize;
    let status =
        unsafe { clankers_engine_run(ptr::null_mut(), ptr::null(), 0, &mut outputs, &mut count) };
    assert_eq!(status, ClankersStatus::NullPointer);
    unsafe {
        let msg = std::ffi::CStr::from_ptr(clankers_last_error_message());
        assert!(!msg.to_bytes().is_empty());
    }
    assert_ne!(clankers_last_error_code(), ClankersStatus::Ok);
}

#[test]
fn noop_identity_roundtrip() {
    let engine = noop_engine([1, 4]);
    assert_eq!(unsafe { clankers_engine_input_count(engine) }, 1);

    let input = vec![1.0f32, 2.0, 3.0, 4.0];
    let view = f32_view(&input, shape_1x4());
    let mut outputs: *mut *mut ClankersTensor = ptr::null_mut();
    let mut count = 0usize;
    assert_eq!(
        unsafe { clankers_engine_run(engine, &view, 1, &mut outputs, &mut count) },
        ClankersStatus::Ok
    );
    assert_eq!(count, 1);
    unsafe {
        let out = *outputs;
        let data = clankers_tensor_data(out);
        let len = clankers_tensor_byte_len(out);
        let slice = std::slice::from_raw_parts(data as *const f32, len / 4);
        assert_eq!(slice, input.as_slice());
        clankers_tensor_destroy(out);
        clankers_output_array_destroy(outputs, count);
    }
    unsafe {
        clankers_engine_destroy(engine);
    }
}

#[test]
fn run_with_stats_reports_zero_copy() {
    let engine = noop_engine([1, 4]);
    let input = vec![5.0f32; 4];
    let view = f32_view(&input, shape_1x4());
    let mut outputs: *mut *mut ClankersTensor = ptr::null_mut();
    let mut count = 0usize;
    let mut stats = ClankersInferenceStats::default();
    assert_eq!(
        unsafe {
            clankers_engine_run_with_stats(engine, &view, 1, &mut outputs, &mut count, &mut stats)
        },
        ClankersStatus::Ok
    );
    assert_eq!(stats.copies, 0);
    assert_eq!(stats.allocations, 1);
    unsafe {
        clankers_tensor_destroy(*outputs);
        clankers_output_array_destroy(outputs, count);
        clankers_engine_destroy(engine);
    }
}

#[test]
fn run_into_zero_alloc_hot_loop() {
    unsafe {
        let builder = clankers_engine_builder_new();
        clankers_engine_builder_set_backend(builder, cstr("noop").as_ptr());
        clankers_engine_builder_set_noop_shape(builder, [1usize, 4].as_ptr(), 2);
        clankers_engine_builder_set_strict_realtime(builder, true);
        let mut engine = ptr::null_mut();
        assert_eq!(
            clankers_engine_builder_build(builder, &mut engine),
            ClankersStatus::Ok
        );

        let input = vec![1.0f32, 2.0, 3.0, 4.0];
        let in_view = f32_view(&input, shape_1x4());
        let mut out_buf = vec![0.0f32; 4];
        let out_shape = shape_1x4();
        let mut out_view = ClankersTensorViewMut {
            data: ptr::null_mut(),
            byte_len: 0,
            dtype: ClankersDType::F32,
            shape: out_shape,
            layout: ClankersLayout::Contiguous,
            device: clankers_ffi::ClankersDevice::Cpu,
        };
        let status = clankers_tensor_view_mut_from_external(
            out_buf.as_mut_ptr() as *mut u8,
            std::mem::size_of_val(out_buf.as_slice()),
            ClankersDType::F32,
            out_shape,
            ClankersLayout::Contiguous,
            &mut out_view,
        );
        assert_eq!(status, ClankersStatus::Ok);

        for i in 0..100 {
            let mut stats = ClankersInferenceStats::default();
            assert_eq!(
                clankers_engine_run_into(engine, &in_view, 1, &mut out_view, 1, &mut stats),
                ClankersStatus::Ok,
                "iteration {i}"
            );
            assert_eq!(stats.allocations, 0, "iteration {i}");
            assert_eq!(stats.copies, 0, "iteration {i}");
        }
        assert_eq!(out_buf, input);
        clankers_engine_destroy(engine);
    }
}

#[cfg(feature = "onnxruntime")]
#[test]
fn onnx_policy_single_f32_loads_and_runs() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../clankers-ml/tests/fixtures/onnx/policy_single_f32.onnx");
    unsafe {
        let builder = clankers_engine_builder_new();
        clankers_engine_builder_set_backend(builder, cstr("onnxruntime").as_ptr());
        clankers_engine_builder_set_model_path(builder, cstr(path.to_str().unwrap()).as_ptr());
        let mut engine = ptr::null_mut();
        assert_eq!(
            clankers_engine_builder_build(builder, &mut engine),
            ClankersStatus::Ok
        );

        let input = vec![0.1f32, 0.2, 0.3, 0.4];
        let view = f32_view(&input, shape_1x4());
        let mut outputs: *mut *mut ClankersTensor = ptr::null_mut();
        let mut count = 0usize;
        assert_eq!(
            clankers_engine_run(engine, &view, 1, &mut outputs, &mut count),
            ClankersStatus::Ok
        );
        assert_eq!(count, 1);
        clankers_tensor_destroy(*outputs);
        clankers_output_array_destroy(outputs, count);
        clankers_engine_destroy(engine);
    }
}

#[test]
fn c_header_compiles() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let header = manifest_dir.join("../../cpp/include/clankers/clankers.h");
    assert!(
        header.exists(),
        "header should be generated at build time: {}",
        header.display()
    );
    let smoke = manifest_dir.join("tests/c_header_smoke.c");
    let out = std::env::temp_dir().join("clankers_c_header_smoke.o");
    let status = std::process::Command::new("cc")
        .arg("-c")
        .arg("-Wall")
        .arg("-Werror")
        .arg("-I")
        .arg(header.parent().unwrap())
        .arg("-o")
        .arg(&out)
        .arg(&smoke)
        .status()
        .expect("invoke cc");
    assert!(status.success(), "C header smoke compile failed");
    let _ = std::fs::remove_file(out);
}
