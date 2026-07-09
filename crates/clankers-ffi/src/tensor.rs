//! Owned tensors and external tensor views.

use std::os::raw::c_char;

use clankers_tensor::{Tensor, TensorView};

use crate::panic::guard;
use crate::types::{device_to_c, dtype_to_c, layout_to_c, shape_to_c};
use crate::{
    error, ffi_arg_check, ffi_null_check, ClankersDType, ClankersLayout, ClankersShape,
    ClankersStatus, ClankersTensor, ClankersTensorView,
};

/// Internal owned tensor storage behind [`ClankersTensor`].
pub(crate) struct TensorHandle {
    pub tensor: Tensor,
}

/// Destroy an owned tensor allocated by clankeRS.
#[no_mangle]
pub extern "C" fn clankers_tensor_destroy(tensor: *mut ClankersTensor) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(tensor);
        unsafe {
            drop(Box::from_raw(tensor as *mut TensorHandle));
        }
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Returns a pointer to the tensor's byte buffer (valid until destroy).
#[no_mangle]
pub extern "C" fn clankers_tensor_data(tensor: *const ClankersTensor) -> *const u8 {
    crate::panic::guard_const_ptr(|| {
        if tensor.is_null() {
            error::set_last_error(ClankersStatus::NullPointer, "tensor is null");
            return std::ptr::null();
        }
        let handle = unsafe { &*(tensor as *const TensorHandle) };
        handle.tensor.bytes().as_ptr()
    })
}

/// Byte length of the tensor buffer.
#[no_mangle]
pub extern "C" fn clankers_tensor_byte_len(tensor: *const ClankersTensor) -> usize {
    crate::panic::guard_value(
        || {
            if tensor.is_null() {
                error::set_last_error(ClankersStatus::NullPointer, "tensor is null");
                return 0;
            }
            let handle = unsafe { &*(tensor as *const TensorHandle) };
            handle.tensor.num_bytes()
        },
        0,
    )
}

/// Element dtype of an owned tensor.
#[no_mangle]
pub extern "C" fn clankers_tensor_dtype(tensor: *const ClankersTensor) -> ClankersDType {
    crate::panic::guard_value(
        || {
            if tensor.is_null() {
                return ClankersDType::F32;
            }
            let handle = unsafe { &*(tensor as *const TensorHandle) };
            dtype_to_c(handle.tensor.dtype())
        },
        ClankersDType::F32,
    )
}

/// Shape of an owned tensor.
#[no_mangle]
pub extern "C" fn clankers_tensor_shape(tensor: *const ClankersTensor) -> ClankersShape {
    crate::panic::guard_value(
        || {
            if tensor.is_null() {
                return ClankersShape {
                    dims: [0; crate::CLANKERS_MAX_RANK],
                    rank: 0,
                };
            }
            let handle = unsafe { &*(tensor as *const TensorHandle) };
            shape_to_c(handle.tensor.shape())
        },
        ClankersShape {
            dims: [0; crate::CLANKERS_MAX_RANK],
            rank: 0,
        },
    )
}

/// Build a borrowed view descriptor over caller-owned memory (zero-copy).
///
/// `data` must remain valid for the lifetime of the inference call.
#[no_mangle]
pub extern "C" fn clankers_tensor_view_from_external(
    data: *const u8,
    byte_len: usize,
    dtype: ClankersDType,
    shape: ClankersShape,
    layout: ClankersLayout,
    out_view: *mut ClankersTensorView,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_view);
        ffi_arg_check!(!data.is_null(), "data pointer is null");
        ffi_arg_check!(byte_len > 0, "byte_len must be > 0");

        let rust_dtype = match crate::types::dtype_from_c(dtype) {
            Some(d) => d,
            None => {
                error::set_last_error(ClankersStatus::InvalidArg, "invalid dtype");
                return ClankersStatus::InvalidArg;
            }
        };
        let rust_layout = match crate::types::layout_from_c(layout) {
            Some(l) => l,
            None => {
                error::set_last_error(ClankersStatus::InvalidArg, "invalid layout");
                return ClankersStatus::InvalidArg;
            }
        };
        let rust_shape = match crate::types::shape_from_c(&shape) {
            Some(s) => s,
            None => {
                error::set_last_error(ClankersStatus::InvalidArg, "invalid shape rank");
                return ClankersStatus::InvalidArg;
            }
        };

        let slice = unsafe { std::slice::from_raw_parts(data, byte_len) };
        if let Err(e) = TensorView::from_slice(slice, rust_dtype, &rust_shape, rust_layout) {
            return error::set_from_tensor(e);
        }

        unsafe {
            *out_view = ClankersTensorView {
                data,
                byte_len,
                dtype,
                shape,
                layout,
                device: crate::ClankersDevice::Cpu,
            };
        }
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Build a borrowed mutable view descriptor over caller-owned memory.
#[no_mangle]
pub extern "C" fn clankers_tensor_view_mut_from_external(
    data: *mut u8,
    byte_len: usize,
    dtype: ClankersDType,
    shape: ClankersShape,
    layout: ClankersLayout,
    out_view: *mut crate::ClankersTensorViewMut,
) -> ClankersStatus {
    guard(|| {
        ffi_null_check!(out_view);
        ffi_arg_check!(!data.is_null(), "data pointer is null");
        ffi_arg_check!(byte_len > 0, "byte_len must be > 0");

        let rust_dtype = match crate::types::dtype_from_c(dtype) {
            Some(d) => d,
            None => {
                error::set_last_error(ClankersStatus::InvalidArg, "invalid dtype");
                return ClankersStatus::InvalidArg;
            }
        };
        let rust_layout = match crate::types::layout_from_c(layout) {
            Some(l) => l,
            None => {
                error::set_last_error(ClankersStatus::InvalidArg, "invalid layout");
                return ClankersStatus::InvalidArg;
            }
        };
        let rust_shape = match crate::types::shape_from_c(&shape) {
            Some(s) => s,
            None => {
                error::set_last_error(ClankersStatus::InvalidArg, "invalid shape rank");
                return ClankersStatus::InvalidArg;
            }
        };

        let slice = unsafe { std::slice::from_raw_parts_mut(data, byte_len) };
        if let Err(e) =
            clankers_tensor::TensorViewMut::from_slice(slice, rust_dtype, rust_shape, rust_layout)
        {
            return error::set_from_tensor(e);
        }

        unsafe {
            *out_view = crate::ClankersTensorViewMut {
                data,
                byte_len,
                dtype,
                shape,
                layout,
                device: crate::ClankersDevice::Cpu,
            };
        }
        error::clear_last_error();
        ClankersStatus::Ok
    })
}

/// Wrap an owned [`Tensor`] as a C handle.
pub(crate) fn tensor_into_raw(tensor: Tensor) -> *mut ClankersTensor {
    Box::into_raw(Box::new(TensorHandle { tensor })) as *mut ClankersTensor
}

/// Borrow an owned tensor as a [`TensorView`].
pub(crate) fn tensor_as_view(handle: &TensorHandle) -> TensorView<'_> {
    handle.tensor.view()
}

/// Fill a [`ClankersTensorView`] from a Rust view (for tests / introspection).
pub(crate) fn view_to_c(view: &TensorView<'_>) -> ClankersTensorView {
    ClankersTensorView {
        data: view.bytes().as_ptr(),
        byte_len: view.num_bytes(),
        dtype: dtype_to_c(view.dtype()),
        shape: shape_to_c(view.shape()),
        layout: layout_to_c(view.layout()),
        device: device_to_c(view.device()),
    }
}

/// Parse dtype name helper (unused but reserved).
#[allow(dead_code)]
fn parse_dtype_name(_name: *const c_char) -> Option<ClankersDType> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use clankers_tensor::{DType, Shape};

    #[test]
    fn external_view_validates_length() {
        let data = [0.0f32; 4];
        let shape = ClankersShape {
            dims: [1, 4, 0, 0, 0, 0],
            rank: 2,
        };
        let mut view = ClankersTensorView {
            data: std::ptr::null(),
            byte_len: 0,
            dtype: ClankersDType::F32,
            shape,
            layout: ClankersLayout::Contiguous,
            device: crate::ClankersDevice::Cpu,
        };
        let status = clankers_tensor_view_from_external(
            data.as_ptr() as *const u8,
            data.len() * std::mem::size_of::<f32>(),
            ClankersDType::F32,
            shape,
            ClankersLayout::Contiguous,
            &mut view,
        );
        assert_eq!(status, ClankersStatus::Ok);
        assert_eq!(view.byte_len, 16);
        let _ = DType::F32;
        let _ = Shape::from([1, 4]);
    }
}
