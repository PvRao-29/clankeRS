//! Inference engine C API.

use std::ffi::CString;
use std::os::raw::c_char;
use std::path::PathBuf;

use clankers_ml::backend::{NoopBackend, NoopSession, TensorSpec};
#[cfg(feature = "onnxruntime")]
use clankers_ml::inference::ModelSource;
use clankers_ml::inference::{InferenceEngine, InferenceEngineBuilder, InferenceStats};
use clankers_tensor::{ShapeSpec, TensorView, TensorViewMut};

#[cfg(feature = "onnxruntime")]
use clankers_ml::backend::{OnnxRuntimeBackend, OnnxSession};

use crate::panic::guard;
use crate::stats::stats_to_c;
use crate::tensor::tensor_into_raw;
use crate::types::{shape_spec_to_c, view_from_c_with_shape, view_mut_from_c_with_shape};
use crate::{
    cstr_to_str, error, ffi_arg_check, ffi_null_check, ffi_try, ClankersEngine,
    ClankersEngineBuilder, ClankersInferenceStats, ClankersNamedInput, ClankersStatus,
    ClankersTensor, ClankersTensorSpec, ClankersTensorView, ClankersTensorViewMut,
    CLANKERS_MAX_RANK,
};

/// Type-erased engine for the C ABI.
enum DynEngine {
    Noop(InferenceEngine<NoopSession>),
    #[cfg(feature = "onnxruntime")]
    Onnx(InferenceEngine<OnnxSession>),
}

impl DynEngine {
    fn input_specs(&self) -> &[TensorSpec] {
        match self {
            DynEngine::Noop(e) => e.input_specs(),
            #[cfg(feature = "onnxruntime")]
            DynEngine::Onnx(e) => e.input_specs(),
        }
    }

    fn output_specs(&self) -> &[TensorSpec] {
        match self {
            DynEngine::Noop(e) => e.output_specs(),
            #[cfg(feature = "onnxruntime")]
            DynEngine::Onnx(e) => e.output_specs(),
        }
    }

    fn run(
        &mut self,
        inputs: &[TensorView<'_>],
    ) -> Result<Vec<clankers_tensor::Tensor>, clankers_ml::inference::InferenceError> {
        match self {
            DynEngine::Noop(e) => e.run(inputs),
            #[cfg(feature = "onnxruntime")]
            DynEngine::Onnx(e) => e.run(inputs),
        }
    }

    fn run_with_stats(
        &mut self,
        inputs: &[TensorView<'_>],
    ) -> Result<
        (Vec<clankers_tensor::Tensor>, InferenceStats),
        clankers_ml::inference::InferenceError,
    > {
        match self {
            DynEngine::Noop(e) => e.run_with_stats(inputs),
            #[cfg(feature = "onnxruntime")]
            DynEngine::Onnx(e) => e.run_with_stats(inputs),
        }
    }

    fn run_named(
        &mut self,
        named: &[(&str, TensorView<'_>)],
    ) -> Result<Vec<clankers_tensor::Tensor>, clankers_ml::inference::InferenceError> {
        match self {
            DynEngine::Noop(e) => e.run_named(named),
            #[cfg(feature = "onnxruntime")]
            DynEngine::Onnx(e) => e.run_named(named),
        }
    }

    fn run_into(
        &mut self,
        inputs: &[TensorView<'_>],
        outputs: &mut [TensorViewMut<'_>],
    ) -> Result<InferenceStats, clankers_ml::inference::InferenceError> {
        match self {
            DynEngine::Noop(e) => e.run_into(inputs, outputs),
            #[cfg(feature = "onnxruntime")]
            DynEngine::Onnx(e) => e.run_into(inputs, outputs),
        }
    }
}

pub(crate) struct EngineHandle {
    engine: DynEngine,
}

enum BackendChoice {
    Noop,
    #[cfg(feature = "onnxruntime")]
    OnnxRuntime,
}

pub(crate) struct BuilderHandle {
    backend: BackendChoice,
    model_path: Option<PathBuf>,
    noop_shape: Vec<usize>,
    warmup: u32,
    strict_realtime: bool,
    profiling: bool,
}

impl Default for BuilderHandle {
    fn default() -> Self {
        BuilderHandle {
            backend: BackendChoice::Noop,
            model_path: None,
            noop_shape: vec![1, 4],
            warmup: 0,
            strict_realtime: false,
            profiling: false,
        }
    }
}

fn engine_mut(engine: *mut ClankersEngine) -> Result<&'static mut EngineHandle, ClankersStatus> {
    if engine.is_null() {
        error::set_last_error(ClankersStatus::NullPointer, "engine is null");
        return Err(ClankersStatus::NullPointer);
    }
    Ok(unsafe { &mut *(engine as *mut EngineHandle) })
}

fn engine_ref(engine: *const ClankersEngine) -> Result<&'static EngineHandle, ClankersStatus> {
    if engine.is_null() {
        error::set_last_error(ClankersStatus::NullPointer, "engine is null");
        return Err(ClankersStatus::NullPointer);
    }
    Ok(unsafe { &*(engine as *const EngineHandle) })
}

fn builder_mut(
    builder: *mut ClankersEngineBuilder,
) -> Result<&'static mut BuilderHandle, ClankersStatus> {
    if builder.is_null() {
        error::set_last_error(ClankersStatus::NullPointer, "builder is null");
        return Err(ClankersStatus::NullPointer);
    }
    Ok(unsafe { &mut *(builder as *mut BuilderHandle) })
}

fn build_engine_from_builder(
    builder: &BuilderHandle,
) -> Result<EngineHandle, clankers_ml::inference::InferenceError> {
    match builder.backend {
        BackendChoice::Noop => {
            let shape = ShapeSpec::from_onnx_dims(
                &builder
                    .noop_shape
                    .iter()
                    .map(|&d| d as i64)
                    .collect::<Vec<_>>(),
            );
            let engine = InferenceEngineBuilder::new(NoopBackend::identity(shape))
                .warmup(builder.warmup)
                .strict_realtime(builder.strict_realtime)
                .build()?;
            Ok(EngineHandle {
                engine: DynEngine::Noop(engine),
            })
        }
        #[cfg(feature = "onnxruntime")]
        BackendChoice::OnnxRuntime => {
            let path = builder.model_path.clone().ok_or_else(|| {
                clankers_ml::inference::InferenceError::Config(
                    "model path required for onnxruntime backend".into(),
                )
            })?;
            let engine = InferenceEngineBuilder::new(OnnxRuntimeBackend::default())
                .model(ModelSource::Path(path))
                .warmup(builder.warmup)
                .strict_realtime(builder.strict_realtime)
                .build()?;
            Ok(EngineHandle {
                engine: DynEngine::Onnx(engine),
            })
        }
    }
}

fn write_output_array(
    outputs: Vec<clankers_tensor::Tensor>,
    out_outputs: *mut *mut *mut ClankersTensor,
    out_output_count: *mut usize,
) {
    let count = outputs.len();
    let vec: Vec<*mut ClankersTensor> = outputs.into_iter().map(tensor_into_raw).collect();
    let raw = vec.into_boxed_slice();
    let ptr = Box::into_raw(raw) as *mut *mut ClankersTensor;
    unsafe {
        *out_outputs = ptr;
        *out_output_count = count;
    }
}

fn fill_tensor_spec(spec: &TensorSpec, out_spec: *mut ClankersTensorSpec) {
    let cname = CString::new(spec.name.as_str()).expect("spec name");
    let name_ptr = cname.into_raw();
    unsafe {
        *out_spec = ClankersTensorSpec {
            name: name_ptr,
            dtype: crate::types::dtype_to_c(spec.dtype),
            shape: shape_spec_to_c(&spec.shape),
            layout: crate::types::layout_to_c(spec.layout),
        };
    }
}

/// Convenience: create an ONNX engine from a model path.
#[no_mangle]
pub extern "C" fn clankers_engine_create(
    model_path: *const c_char,
    out_engine: *mut *mut ClankersEngine,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_engine);
        #[cfg(not(feature = "onnxruntime"))]
        {
            let _ = model_path;
            error::set_last_error(
                ClankersStatus::Unsupported,
                "onnxruntime backend not enabled",
            );
            return ClankersStatus::Unsupported;
        }
        #[cfg(feature = "onnxruntime")]
        {
            let path = match cstr_to_str(model_path) {
                Ok(s) => s,
                Err(s) => return s,
            };
            let builder = BuilderHandle {
                backend: BackendChoice::OnnxRuntime,
                model_path: Some(PathBuf::from(path)),
                ..BuilderHandle::default()
            };
            let handle = match build_engine_from_builder(&builder) {
                Ok(h) => h,
                Err(e) => return error::set_from_inference(e),
            };
            let raw = Box::into_raw(Box::new(handle)) as *mut ClankersEngine;
            unsafe {
                *out_engine = raw;
            }
            error::clear_last_error();
            ClankersStatus::Ok
        }
    })
}

/// Create a new engine builder.
#[no_mangle]
pub extern "C" fn clankers_engine_builder_new() -> *mut ClankersEngineBuilder {
    crate::panic::guard_ptr(|| {
        let handle = Box::new(BuilderHandle::default());
        Box::into_raw(handle) as *mut ClankersEngineBuilder
    })
}

/// Destroy an engine builder (safe on null).
#[no_mangle]
pub extern "C" fn clankers_engine_builder_destroy(builder: *mut ClankersEngineBuilder) {
    let _ = guard(|| {
        if !builder.is_null() {
            unsafe {
                drop(Box::from_raw(builder as *mut BuilderHandle));
            }
        }
        error::clear_last_error();
        ClankersStatus::Ok
    });
}

/// Set the model file path (required for `onnxruntime` backend).
#[no_mangle]
pub extern "C" fn clankers_engine_builder_set_model_path(
    builder: *mut ClankersEngineBuilder,
    path: *const c_char,
) -> ClankersStatus {
    guard(|| {
        let b = match builder_mut(builder) {
            Ok(b) => b,
            Err(s) => return s,
        };
        let path_str = match cstr_to_str(path) {
            Ok(s) => s,
            Err(s) => return s,
        };
        b.model_path = Some(PathBuf::from(path_str));
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Select backend: `"noop"` or `"onnxruntime"`.
#[no_mangle]
pub extern "C" fn clankers_engine_builder_set_backend(
    builder: *mut ClankersEngineBuilder,
    backend: *const c_char,
) -> ClankersStatus {
    guard(|| {
        let b = match builder_mut(builder) {
            Ok(b) => b,
            Err(s) => return s,
        };
        let name = match cstr_to_str(backend) {
            Ok(s) => s,
            Err(s) => return s,
        };
        b.backend = match name {
            "noop" => BackendChoice::Noop,
            "onnxruntime" => {
                #[cfg(not(feature = "onnxruntime"))]
                {
                    error::set_last_error(
                        ClankersStatus::Unsupported,
                        "onnxruntime backend not enabled in this build",
                    );
                    return ClankersStatus::Unsupported;
                }
                #[cfg(feature = "onnxruntime")]
                {
                    BackendChoice::OnnxRuntime
                }
            }
            other => {
                error::set_last_error(
                    ClankersStatus::InvalidArg,
                    format!("unknown backend {other:?}"),
                );
                return ClankersStatus::InvalidArg;
            }
        };
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Configure the identity shape used by the `noop` backend (default `[1, 4]`).
#[no_mangle]
pub extern "C" fn clankers_engine_builder_set_noop_shape(
    builder: *mut ClankersEngineBuilder,
    dims: *const usize,
    rank: usize,
) -> ClankersStatus {
    guard(|| {
        let b = match builder_mut(builder) {
            Ok(b) => b,
            Err(s) => return s,
        };
        ffi_null_check!(dims);
        ffi_arg_check!(rank <= CLANKERS_MAX_RANK, "rank exceeds CLANKERS_MAX_RANK");
        let slice = unsafe { std::slice::from_raw_parts(dims, rank) };
        b.noop_shape = slice.to_vec();
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Enable strict real-time checks at build time.
#[no_mangle]
pub extern "C" fn clankers_engine_builder_set_strict_realtime(
    builder: *mut ClankersEngineBuilder,
    enabled: bool,
) -> ClankersStatus {
    guard(|| {
        let b = match builder_mut(builder) {
            Ok(b) => b,
            Err(s) => return s,
        };
        b.strict_realtime = enabled;
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Reserved profiling toggle (stats are always available via run_with_stats).
#[no_mangle]
pub extern "C" fn clankers_engine_builder_set_profiling(
    builder: *mut ClankersEngineBuilder,
    enabled: bool,
) -> ClankersStatus {
    guard(|| {
        let b = match builder_mut(builder) {
            Ok(b) => b,
            Err(s) => return s,
        };
        b.profiling = enabled;
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Set warm-up run count at build time.
#[no_mangle]
pub extern "C" fn clankers_engine_builder_set_warmup(
    builder: *mut ClankersEngineBuilder,
    runs: u32,
) -> ClankersStatus {
    guard(|| {
        let b = match builder_mut(builder) {
            Ok(b) => b,
            Err(s) => return s,
        };
        b.warmup = runs;
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Build an engine from a configured builder. On success, `*out_engine` is set
/// and the builder is consumed (do not destroy it).
#[no_mangle]
pub extern "C" fn clankers_engine_builder_build(
    builder: *mut ClankersEngineBuilder,
    out_engine: *mut *mut ClankersEngine,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_engine);
        let b = match builder_mut(builder) {
            Ok(b) => b,
            Err(s) => return s,
        };
        let config = BuilderHandle {
            backend: match b.backend {
                BackendChoice::Noop => BackendChoice::Noop,
                #[cfg(feature = "onnxruntime")]
                BackendChoice::OnnxRuntime => BackendChoice::OnnxRuntime,
            },
            model_path: b.model_path.clone(),
            noop_shape: b.noop_shape.clone(),
            warmup: b.warmup,
            strict_realtime: b.strict_realtime,
            profiling: b.profiling,
        };
        let handle = match build_engine_from_builder(&config) {
            Ok(e) => e,
            Err(e) => return error::set_from_inference(e),
        };
        let raw = Box::into_raw(Box::new(handle)) as *mut ClankersEngine;
        unsafe {
            *out_engine = raw;
            let _ = Box::from_raw(builder as *mut BuilderHandle);
        }
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Destroy an engine.
#[no_mangle]
pub extern "C" fn clankers_engine_destroy(engine: *mut ClankersEngine) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(engine);
        unsafe {
            drop(Box::from_raw(engine as *mut EngineHandle));
        }
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Number of model inputs.
#[no_mangle]
pub extern "C" fn clankers_engine_input_count(engine: *const ClankersEngine) -> usize {
    crate::panic::guard_value(
        || match engine_ref(engine) {
            Ok(h) => h.engine.input_specs().len(),
            Err(_) => 0,
        },
        0,
    )
}

/// Number of model outputs.
#[no_mangle]
pub extern "C" fn clankers_engine_output_count(engine: *const ClankersEngine) -> usize {
    crate::panic::guard_value(
        || match engine_ref(engine) {
            Ok(h) => h.engine.output_specs().len(),
            Err(_) => 0,
        },
        0,
    )
}

/// Fill `out_spec` with the input tensor spec at `index`.
///
/// Free `out_spec.name` with [`clankers_tensor_spec_destroy`] when done.
#[no_mangle]
pub extern "C" fn clankers_engine_input_spec(
    engine: *const ClankersEngine,
    index: usize,
    out_spec: *mut ClankersTensorSpec,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_spec);
        let handle = match engine_ref(engine) {
            Ok(h) => h,
            Err(s) => return s,
        };
        let spec = match handle.engine.input_specs().get(index) {
            Some(s) => s,
            None => {
                error::set_last_error(ClankersStatus::InvalidArg, "input index out of range");
                return ClankersStatus::InvalidArg;
            }
        };
        fill_tensor_spec(spec, out_spec);
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Fill `out_spec` with the output tensor spec at `index`.
#[no_mangle]
pub extern "C" fn clankers_engine_output_spec(
    engine: *const ClankersEngine,
    index: usize,
    out_spec: *mut ClankersTensorSpec,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_spec);
        let handle = match engine_ref(engine) {
            Ok(h) => h,
            Err(s) => return s,
        };
        let spec = match handle.engine.output_specs().get(index) {
            Some(s) => s,
            None => {
                error::set_last_error(ClankersStatus::InvalidArg, "output index out of range");
                return ClankersStatus::InvalidArg;
            }
        };
        fill_tensor_spec(spec, out_spec);
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Free the `name` field allocated by spec query functions.
#[no_mangle]
pub extern "C" fn clankers_tensor_spec_destroy(spec: *mut ClankersTensorSpec) {
    if spec.is_null() {
        return;
    }
    let _ = guard(|| {
        unsafe {
            let s = &mut *spec;
            if !s.name.is_null() {
                let _ = CString::from_raw(s.name as *mut c_char);
                s.name = std::ptr::null();
            }
        }
        ClankersStatus::Ok
    });
}

/// Run inference. Caller must destroy each output tensor and the output array.
#[no_mangle]
pub extern "C" fn clankers_engine_run(
    engine: *mut ClankersEngine,
    inputs: *const ClankersTensorView,
    input_count: usize,
    out_outputs: *mut *mut *mut ClankersTensor,
    out_output_count: *mut usize,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_outputs);
        ffi_null_check!(out_output_count);
        let handle = match engine_mut(engine) {
            Ok(h) => h,
            Err(s) => return s,
        };
        if input_count > 0 {
            ffi_null_check!(inputs);
        }
        let c_inputs = if input_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(inputs, input_count) }
        };
        let mut shapes = Vec::with_capacity(input_count);
        for c_view in c_inputs {
            match crate::types::shape_from_c(&c_view.shape) {
                Some(s) => shapes.push(s),
                None => {
                    error::set_last_error(ClankersStatus::InvalidArg, "invalid input shape");
                    return ClankersStatus::InvalidArg;
                }
            }
        }
        let mut rust_views = Vec::with_capacity(input_count);
        for (c_view, shape) in c_inputs.iter().zip(&shapes) {
            let view = match unsafe { view_from_c_with_shape(c_view, shape) } {
                Ok(v) => v,
                Err(s) => return s,
            };
            rust_views.push(view);
        }
        let outputs = ffi_try!(handle.engine.run(&rust_views));
        write_output_array(outputs, out_outputs, out_output_count);
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Run inference and return per-run stats.
#[no_mangle]
pub extern "C" fn clankers_engine_run_with_stats(
    engine: *mut ClankersEngine,
    inputs: *const ClankersTensorView,
    input_count: usize,
    out_outputs: *mut *mut *mut ClankersTensor,
    out_output_count: *mut usize,
    out_stats: *mut ClankersInferenceStats,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_outputs);
        ffi_null_check!(out_output_count);
        ffi_null_check!(out_stats);
        let handle = match engine_mut(engine) {
            Ok(h) => h,
            Err(s) => return s,
        };
        if input_count > 0 {
            ffi_null_check!(inputs);
        }
        let c_inputs = if input_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(inputs, input_count) }
        };
        let mut shapes = Vec::with_capacity(input_count);
        for c_view in c_inputs {
            match crate::types::shape_from_c(&c_view.shape) {
                Some(s) => shapes.push(s),
                None => {
                    error::set_last_error(ClankersStatus::InvalidArg, "invalid input shape");
                    return ClankersStatus::InvalidArg;
                }
            }
        }
        let mut rust_views = Vec::with_capacity(input_count);
        for (c_view, shape) in c_inputs.iter().zip(&shapes) {
            let view = match unsafe { view_from_c_with_shape(c_view, shape) } {
                Ok(v) => v,
                Err(s) => return s,
            };
            rust_views.push(view);
        }
        let (outputs, stats) = ffi_try!(handle.engine.run_with_stats(&rust_views));
        write_output_array(outputs, out_outputs, out_output_count);
        unsafe {
            *out_stats = stats_to_c(&stats);
        }
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Free the output pointer array returned by run functions (not the tensors).
#[no_mangle]
pub extern "C" fn clankers_output_array_destroy(outputs: *mut *mut ClankersTensor, count: usize) {
    if outputs.is_null() || count == 0 {
        return;
    }
    unsafe {
        let slice_ptr = std::ptr::slice_from_raw_parts_mut(outputs, count);
        let _ = Box::from_raw(slice_ptr);
    }
}

/// Run inference with named inputs (any order).
#[no_mangle]
pub extern "C" fn clankers_engine_run_named(
    engine: *mut ClankersEngine,
    named: *const ClankersNamedInput,
    named_count: usize,
    out_outputs: *mut *mut *mut ClankersTensor,
    out_output_count: *mut usize,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_outputs);
        ffi_null_check!(out_output_count);
        let handle = match engine_mut(engine) {
            Ok(h) => h,
            Err(s) => return s,
        };
        if named_count > 0 {
            ffi_null_check!(named);
        }
        let slice = if named_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(named, named_count) }
        };
        let mut shapes = Vec::with_capacity(named_count);
        for entry in slice {
            match crate::types::shape_from_c(&entry.view.shape) {
                Some(s) => shapes.push(s),
                None => {
                    error::set_last_error(ClankersStatus::InvalidArg, "invalid input shape");
                    return ClankersStatus::InvalidArg;
                }
            }
        }
        let mut pairs = Vec::with_capacity(named_count);
        for (entry, shape) in slice.iter().zip(&shapes) {
            let name = match cstr_to_str(entry.name) {
                Ok(s) => s,
                Err(s) => return s,
            };
            let view = match unsafe { view_from_c_with_shape(&entry.view, shape) } {
                Ok(v) => v,
                Err(s) => return s,
            };
            pairs.push((name, view));
        }
        let outputs = ffi_try!(handle.engine.run_named(&pairs));
        write_output_array(outputs, out_outputs, out_output_count);
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Run inference into caller-preallocated output buffers (zero-alloc hot loop).
#[no_mangle]
pub extern "C" fn clankers_engine_run_into(
    engine: *mut ClankersEngine,
    inputs: *const ClankersTensorView,
    input_count: usize,
    outputs: *mut ClankersTensorViewMut,
    output_count: usize,
    out_stats: *mut ClankersInferenceStats,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_stats);
        let handle = match engine_mut(engine) {
            Ok(h) => h,
            Err(s) => return s,
        };
        if input_count > 0 {
            ffi_null_check!(inputs);
        }
        if output_count > 0 {
            ffi_null_check!(outputs);
        }
        let c_inputs = if input_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(inputs, input_count) }
        };
        let mut in_shapes = Vec::with_capacity(input_count);
        for c_view in c_inputs {
            match crate::types::shape_from_c(&c_view.shape) {
                Some(s) => in_shapes.push(s),
                None => {
                    error::set_last_error(ClankersStatus::InvalidArg, "invalid input shape");
                    return ClankersStatus::InvalidArg;
                }
            }
        }
        let mut in_views = Vec::with_capacity(input_count);
        for (c_view, shape) in c_inputs.iter().zip(&in_shapes) {
            let view = match unsafe { view_from_c_with_shape(c_view, shape) } {
                Ok(v) => v,
                Err(s) => return s,
            };
            in_views.push(view);
        }
        let c_outputs = if output_count == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(outputs, output_count) }
        };
        let mut out_shapes = Vec::with_capacity(output_count);
        for c_view in c_outputs.iter() {
            match crate::types::shape_from_c(&c_view.shape) {
                Some(s) => out_shapes.push(s),
                None => {
                    error::set_last_error(ClankersStatus::InvalidArg, "invalid output shape");
                    return ClankersStatus::InvalidArg;
                }
            }
        }
        let mut out_views = Vec::with_capacity(output_count);
        for (c_view, shape) in c_outputs.iter().zip(&out_shapes) {
            let view = match unsafe { view_mut_from_c_with_shape(c_view, shape.clone()) } {
                Ok(v) => v,
                Err(s) => return s,
            };
            out_views.push(view);
        }
        let stats = ffi_try!(handle.engine.run_into(&in_views, &mut out_views));
        unsafe {
            *out_stats = stats_to_c(&stats);
        }
        error::clear_last_error();
        ClankersStatus::Ok
    })
}
