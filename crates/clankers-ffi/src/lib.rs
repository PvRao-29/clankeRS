//! Stable C ABI for clankeRS inference.

#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(dead_code)]

mod engine;
mod error;
mod ffi_types;
mod panic;
mod stats;
mod tensor;
mod types;

pub use ffi_types::*;

use std::ffi::CStr;
use std::os::raw::c_char;

/// clankeRS release version string (e.g. `"0.1.4"`).
pub const CLANKERS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ABI version — bump when breaking the C API layout or semantics.
pub const CLANKERS_ABI_VERSION: u32 = 1;

/// Returns the clankeRS release version string.
#[no_mangle]
pub extern "C" fn clankers_version() -> *const c_char {
    static VERSION: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();
    VERSION
        .get_or_init(|| {
            std::ffi::CString::new(CLANKERS_VERSION).expect("version is valid C string")
        })
        .as_ptr()
}

/// Returns the stable C ABI version number.
#[no_mangle]
pub extern "C" fn clankers_abi_version() -> u32 {
    CLANKERS_ABI_VERSION
}

/// Returns the last error code set on this thread.
#[no_mangle]
pub extern "C" fn clankers_last_error_code() -> ClankersStatus {
    error::last_error_code()
}

/// Returns a human-readable message for the last error on this thread.
///
/// The pointer is valid until the next FFI call on the same thread.
#[no_mangle]
pub extern "C" fn clankers_last_error_message() -> *const c_char {
    error::last_error_message_ptr()
}

/// Frees a string allocated by clankeRS (reserved for future use).
#[no_mangle]
pub extern "C" fn clankers_string_destroy(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(ptr));
        }
    }
}

/// Validate that a C string pointer is readable (helper for tests).
#[doc(hidden)]
pub fn cstr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, ClankersStatus> {
    if ptr.is_null() {
        return Err(ClankersStatus::NullPointer);
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|_| ClankersStatus::InvalidArg)
}
