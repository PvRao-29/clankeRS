//! Thread-local error state (errno-style).

use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;

use crate::ClankersStatus;
use clankers_ml::inference::InferenceError;
use clankers_tensor::TensorError;

thread_local! {
    static LAST_ERROR: RefCell<(ClankersStatus, CString)> = RefCell::new((
        ClankersStatus::Ok,
        CString::new("").expect("empty"),
    ));
}

pub(crate) fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = (ClankersStatus::Ok, CString::new("").expect("empty"));
    });
}

pub(crate) fn set_last_error(status: ClankersStatus, message: impl Into<String>) {
    let msg = CString::new(message.into())
        .unwrap_or_else(|_| CString::new("invalid error message").expect("fallback"));
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = (status, msg);
    });
}

pub(crate) fn set_from_inference(err: InferenceError) -> ClankersStatus {
    let status = map_inference_error(&err);
    set_last_error(status, err.to_string());
    status
}

pub(crate) fn set_from_tensor(err: TensorError) -> ClankersStatus {
    let status = ClankersStatus::InvalidArg;
    set_last_error(status, err.to_string());
    status
}

fn map_inference_error(err: &InferenceError) -> ClankersStatus {
    match err {
        InferenceError::Tensor(_) => ClankersStatus::InvalidArg,
        InferenceError::InvalidInput { .. } => ClankersStatus::InvalidInput,
        InferenceError::InputCount { .. } => ClankersStatus::InvalidInput,
        InferenceError::UnknownInput { .. } => ClankersStatus::InvalidInput,
        InferenceError::InvalidOutput { .. } => ClankersStatus::InvalidInput,
        InferenceError::UnsupportedDevice(_) => ClankersStatus::Unsupported,
        InferenceError::RealtimeUnsatisfiable(_) => ClankersStatus::Config,
        InferenceError::Backend { .. } => ClankersStatus::Backend,
        InferenceError::ModelLoad { .. } => ClankersStatus::ModelLoad,
        InferenceError::UnsupportedPreallocatedOutputs { .. } => ClankersStatus::Unsupported,
        InferenceError::Config(_) => ClankersStatus::Config,
    }
}

pub fn last_error_code() -> ClankersStatus {
    LAST_ERROR.with(|e| e.borrow().0)
}

pub fn last_error_message_ptr() -> *const c_char {
    LAST_ERROR.with(|e| e.borrow().1.as_ptr())
}

#[macro_export]
macro_rules! ffi_try {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => return $crate::error::set_from_inference(e),
        }
    };
}

#[macro_export]
macro_rules! ffi_null_check {
    ($ptr:expr) => {
        if $ptr.is_null() {
            $crate::error::set_last_error(
                $crate::ClankersStatus::NullPointer,
                concat!(stringify!($ptr), " is null"),
            );
            return $crate::ClankersStatus::NullPointer;
        }
    };
}

#[macro_export]
macro_rules! ffi_null_check_ptr {
    ($ptr:expr) => {
        if $ptr.is_null() {
            $crate::error::set_last_error(
                $crate::ClankersStatus::NullPointer,
                concat!(stringify!($ptr), " is null"),
            );
            return std::ptr::null_mut();
        }
    };
}

#[macro_export]
macro_rules! ffi_arg_check {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            $crate::error::set_last_error($crate::ClankersStatus::InvalidArg, $msg);
            return $crate::ClankersStatus::InvalidArg;
        }
    };
}
