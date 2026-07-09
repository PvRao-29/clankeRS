//! Conversions between Rust tensor types and C POD structs.

use clankers_ml::backend::TensorSpec;
use clankers_tensor::{DType, Device, Layout, Shape, ShapeSpec, TensorView, TensorViewMut};

use crate::{
    ClankersDType, ClankersDevice, ClankersLayout, ClankersShape, ClankersTensorSpec,
    CLANKERS_DIM_DYNAMIC, CLANKERS_MAX_RANK,
};

pub fn dtype_to_c(dtype: DType) -> ClankersDType {
    match dtype {
        DType::Bool => ClankersDType::Bool,
        DType::U8 => ClankersDType::U8,
        DType::F16 => ClankersDType::F16,
        DType::F32 => ClankersDType::F32,
        DType::F64 => ClankersDType::F64,
        DType::I32 => ClankersDType::I32,
        DType::I64 => ClankersDType::I64,
    }
}

pub fn dtype_from_c(dtype: ClankersDType) -> Option<DType> {
    match dtype {
        ClankersDType::Bool => Some(DType::Bool),
        ClankersDType::U8 => Some(DType::U8),
        ClankersDType::F16 => Some(DType::F16),
        ClankersDType::F32 => Some(DType::F32),
        ClankersDType::F64 => Some(DType::F64),
        ClankersDType::I32 => Some(DType::I32),
        ClankersDType::I64 => Some(DType::I64),
    }
}

pub fn layout_to_c(layout: Layout) -> ClankersLayout {
    match layout {
        Layout::Contiguous => ClankersLayout::Contiguous,
        Layout::Strided => ClankersLayout::Strided,
    }
}

pub fn layout_from_c(layout: ClankersLayout) -> Option<Layout> {
    match layout {
        ClankersLayout::Contiguous => Some(Layout::Contiguous),
        ClankersLayout::Strided => Some(Layout::Strided),
    }
}

pub fn device_to_c(device: Device) -> ClankersDevice {
    match device {
        Device::Cpu => ClankersDevice::Cpu,
        Device::Cuda(_) => ClankersDevice::Cuda,
        Device::Metal => ClankersDevice::Metal,
    }
}

pub fn shape_to_c(shape: &Shape) -> ClankersShape {
    let mut dims = [0usize; CLANKERS_MAX_RANK];
    let rank = shape.rank().min(CLANKERS_MAX_RANK);
    dims[..rank].copy_from_slice(&shape.dims()[..rank]);
    ClankersShape { dims, rank }
}

pub fn shape_spec_to_c(spec: &ShapeSpec) -> ClankersShape {
    let mut dims = [0usize; CLANKERS_MAX_RANK];
    let rank = spec.rank().min(CLANKERS_MAX_RANK);
    for (i, dim) in spec.dims().iter().take(rank).enumerate() {
        dims[i] = match dim {
            clankers_tensor::Dim::Fixed(n) => *n,
            clankers_tensor::Dim::Dynamic => CLANKERS_DIM_DYNAMIC,
        };
    }
    ClankersShape { dims, rank }
}

pub fn shape_from_c(c_shape: &ClankersShape) -> Option<Shape> {
    if c_shape.rank > CLANKERS_MAX_RANK {
        return None;
    }
    let dims: Vec<usize> = c_shape.dims[..c_shape.rank].to_vec();
    Some(Shape::from(dims))
}

pub fn tensor_spec_to_c(spec: &TensorSpec, name_storage: &str) -> ClankersTensorSpec {
    ClankersTensorSpec {
        name: name_storage.as_ptr() as *const _,
        dtype: dtype_to_c(spec.dtype),
        shape: shape_spec_to_c(&spec.shape),
        layout: layout_to_c(spec.layout),
    }
}

/// Build a [`TensorView`] from a C view using a caller-provided shape buffer.
pub unsafe fn view_from_c_with_shape<'a>(
    view: &crate::ClankersTensorView,
    shape: &'a Shape,
) -> Result<TensorView<'a>, crate::ClankersStatus> {
    if view.data.is_null() {
        return Err(crate::ClankersStatus::NullPointer);
    }
    let dtype = dtype_from_c(view.dtype).ok_or(crate::ClankersStatus::InvalidArg)?;
    let layout = layout_from_c(view.layout).ok_or(crate::ClankersStatus::InvalidArg)?;
    let data = std::slice::from_raw_parts(view.data, view.byte_len);
    TensorView::from_slice(data, dtype, shape, layout).map_err(|e| {
        crate::error::set_from_tensor(e);
        crate::ClankersStatus::InvalidArg
    })
}

/// Build a [`TensorViewMut`] from a C mutable view.
pub unsafe fn view_mut_from_c_with_shape<'a>(
    view: &crate::ClankersTensorViewMut,
    shape: Shape,
) -> Result<TensorViewMut<'a>, crate::ClankersStatus> {
    if view.data.is_null() {
        return Err(crate::ClankersStatus::NullPointer);
    }
    let dtype = dtype_from_c(view.dtype).ok_or(crate::ClankersStatus::InvalidArg)?;
    let layout = layout_from_c(view.layout).ok_or(crate::ClankersStatus::InvalidArg)?;
    let data = std::slice::from_raw_parts_mut(view.data, view.byte_len);
    TensorViewMut::from_slice(data, dtype, shape, layout).map_err(|e| {
        crate::error::set_from_tensor(e);
        crate::ClankersStatus::InvalidArg
    })
}
