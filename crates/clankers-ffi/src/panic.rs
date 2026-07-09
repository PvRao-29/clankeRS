//! Panic guards for every `extern "C"` entry point.

use crate::{error, ClankersStatus};

/// Run an FFI body, mapping panics to [`ClankersStatus::Internal`].
pub fn guard<F>(f: F) -> ClankersStatus
where
    F: FnOnce() -> ClankersStatus + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(status) => status,
        Err(_) => {
            error::set_last_error(ClankersStatus::Internal, "internal panic");
            ClankersStatus::Internal
        }
    }
}

/// Like [`guard`], but for functions that return a pointer.
pub fn guard_ptr<F, T>(f: F) -> *mut T
where
    F: FnOnce() -> *mut T + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(ptr) => ptr,
        Err(_) => {
            error::set_last_error(ClankersStatus::Internal, "internal panic");
            std::ptr::null_mut()
        }
    }
}

/// Like [`guard`], but for functions that return `*const T`.
pub fn guard_const_ptr<F, T>(f: F) -> *const T
where
    F: FnOnce() -> *const T + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(ptr) => ptr,
        Err(_) => {
            error::set_last_error(ClankersStatus::Internal, "internal panic");
            std::ptr::null()
        }
    }
}

/// Like [`guard`], but for functions that return a scalar.
pub fn guard_value<F, T>(f: F, on_panic: T) -> T
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(v) => v,
        Err(_) => {
            error::set_last_error(ClankersStatus::Internal, "internal panic");
            on_panic
        }
    }
}
