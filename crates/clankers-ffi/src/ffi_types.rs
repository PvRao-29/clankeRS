//! C-visible POD types and opaque handle forward declarations.

use std::os::raw::c_char;

/// Maximum tensor rank exposed through the C API.
pub const CLANKERS_MAX_RANK: usize = 6;

/// Sentinel for a dynamic (unknown-at-load-time) dimension in [`ClankersShape`].
pub const CLANKERS_DIM_DYNAMIC: usize = usize::MAX;

/// Opaque inference engine handle.
#[repr(C)]
pub struct ClankersEngine {
    _private: [u8; 0],
}

/// Opaque owned tensor handle returned from `clankers_engine_run`.
#[repr(C)]
pub struct ClankersTensor {
    _private: [u8; 0],
}

/// Opaque engine builder handle.
#[repr(C)]
pub struct ClankersEngineBuilder {
    _private: [u8; 0],
}

/// Element dtype.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClankersDType {
    Bool = 0,
    U8 = 1,
    F16 = 2,
    F32 = 3,
    F64 = 4,
    I32 = 5,
    I64 = 6,
}

/// Memory layout of a tensor buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClankersLayout {
    Contiguous = 0,
    Strided = 1,
}

/// Device placement.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClankersDevice {
    Cpu = 0,
    Cuda = 1,
    Metal = 2,
}

/// Function status / error code.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClankersStatus {
    Ok = 0,
    InvalidArg = 1,
    ModelLoad = 2,
    InvalidInput = 3,
    Backend = 4,
    Allocation = 5,
    NullPointer = 6,
    Internal = 7,
    Unsupported = 8,
    Config = 9,
}

/// Concrete tensor shape.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClankersShape {
    pub dims: [usize; CLANKERS_MAX_RANK],
    pub rank: usize,
}

/// Borrowed read-only tensor view (caller owns `data`).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClankersTensorView {
    pub data: *const u8,
    pub byte_len: usize,
    pub dtype: ClankersDType,
    pub shape: ClankersShape,
    pub layout: ClankersLayout,
    pub device: ClankersDevice,
}

/// Borrowed writable tensor view (caller owns `data`).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClankersTensorViewMut {
    pub data: *mut u8,
    pub byte_len: usize,
    pub dtype: ClankersDType,
    pub shape: ClankersShape,
    pub layout: ClankersLayout,
    pub device: ClankersDevice,
}

/// A named input for `clankers_engine_run_named`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClankersNamedInput {
    pub name: *const c_char,
    pub view: ClankersTensorView,
}

/// Description of one model input or output tensor.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClankersTensorSpec {
    pub name: *const c_char,
    pub dtype: ClankersDType,
    pub shape: ClankersShape,
    pub layout: ClankersLayout,
}

/// Per-run inference accounting.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ClankersInferenceStats {
    pub latency_us: u64,
    pub copies: usize,
    pub bytes_copied: usize,
    pub allocations: usize,
    pub bytes_allocated: usize,
    pub backend_latency_us: u64,
    pub backend_copies: usize,
    pub backend_bytes_copied: usize,
}
