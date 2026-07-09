//! Borrowed tensor views — the zero-copy currency of the inference engine.
//!
//! A [`TensorView`] is a typed, shaped window onto memory the caller already
//! owns (a decoded camera frame, a state vector, an arena slot). Constructing one
//! never allocates and never copies; it only validates that the byte buffer is
//! consistent with the declared [`DType`], [`Shape`], and [`Layout`].

use crate::error::{TensorError, TensorResult};
use crate::{DType, Device, Layout, Shape};

/// Validate that `data` can hold `shape` elements of `dtype` and is aligned.
fn validate(data_len: usize, ptr: usize, dtype: DType, shape: &Shape) -> TensorResult<()> {
    let expected = shape.num_bytes(dtype.element_size());
    if data_len != expected {
        return Err(TensorError::ByteLength {
            dtype,
            shape: shape.clone(),
            expected,
            actual: data_len,
        });
    }
    let align = dtype.element_size();
    if !ptr.is_multiple_of(align) {
        return Err(TensorError::Alignment { dtype, align, ptr });
    }
    Ok(())
}

/// Reinterpret a byte slice as `&[T]`. The caller must have validated that the
/// pointer is aligned for `T` and that `bytes.len()` is a multiple of `size_of::<T>()`.
///
/// # Safety
/// `bytes` must be aligned to `align_of::<T>()` and its length a whole multiple
/// of `size_of::<T>()`. `T` must be a plain-old-data numeric type.
unsafe fn cast_slice<T>(bytes: &[u8]) -> &[T] {
    std::slice::from_raw_parts(bytes.as_ptr() as *const T, bytes.len() / size_of::<T>())
}

/// A borrowed, read-only tensor.
#[derive(Debug, Clone, Copy)]
pub struct TensorView<'a> {
    data: &'a [u8],
    dtype: DType,
    shape: &'a Shape,
    layout: Layout,
    device: Device,
}

impl<'a> TensorView<'a> {
    /// Build a view over raw bytes, validating length and alignment.
    ///
    /// The `shape` is borrowed so repeated per-frame views over a fixed shape do
    /// not reallocate it.
    pub fn from_slice(
        data: &'a [u8],
        dtype: DType,
        shape: &'a Shape,
        layout: Layout,
    ) -> TensorResult<Self> {
        validate(data.len(), data.as_ptr() as usize, dtype, shape)?;
        Ok(TensorView {
            data,
            dtype,
            shape,
            layout,
            device: Device::Cpu,
        })
    }

    /// Build a contiguous `F32` view over a typed slice (the common case).
    pub fn from_f32(data: &'a [f32], shape: &'a Shape) -> TensorResult<Self> {
        // A `&[f32]` is always 4-byte aligned and `len*4` bytes long, so the only
        // thing that can fail here is a shape/length disagreement.
        let bytes = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };
        Self::from_slice(bytes, DType::F32, shape, Layout::Contiguous)
    }

    /// The raw bytes.
    pub fn bytes(&self) -> &'a [u8] {
        self.data
    }

    /// Element dtype.
    pub fn dtype(&self) -> DType {
        self.dtype
    }

    /// Tensor shape.
    pub fn shape(&self) -> &Shape {
        self.shape
    }

    /// Memory layout.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Device placement (always CPU for a borrowed host view).
    pub fn device(&self) -> Device {
        self.device
    }

    /// Number of elements.
    pub fn num_elements(&self) -> usize {
        self.shape.num_elements()
    }

    /// Byte length.
    pub fn num_bytes(&self) -> usize {
        self.data.len()
    }

    /// Whether this view is a dense row-major `F32` block — the shape a typical
    /// ONNX backend can bind without any conversion.
    pub fn is_contiguous(&self) -> bool {
        self.layout.is_contiguous()
    }

    /// View the data as `&[f32]`, if the dtype matches.
    pub fn as_f32(&self) -> TensorResult<&'a [f32]> {
        self.typed::<f32>(DType::F32)
    }

    /// View the data as `&[i64]`, if the dtype matches.
    pub fn as_i64(&self) -> TensorResult<&'a [i64]> {
        self.typed::<i64>(DType::I64)
    }

    /// View the data as `&[u8]`, if the dtype matches.
    pub fn as_u8(&self) -> TensorResult<&'a [u8]> {
        if self.dtype != DType::U8 {
            return Err(TensorError::DTypeMismatch {
                actual: self.dtype,
                requested: DType::U8,
            });
        }
        Ok(self.data)
    }

    fn typed<T>(&self, want: DType) -> TensorResult<&'a [T]> {
        if self.dtype != want {
            return Err(TensorError::DTypeMismatch {
                actual: self.dtype,
                requested: want,
            });
        }
        // SAFETY: `from_slice`/`from_f32` validated alignment for this dtype and
        // that the byte length is an exact multiple of the element size.
        Ok(unsafe { cast_slice::<T>(self.data) })
    }
}

/// A borrowed, writable tensor — used to bind a caller-owned output buffer so
/// inference can write results in place without allocating.
#[derive(Debug)]
pub struct TensorViewMut<'a> {
    data: &'a mut [u8],
    dtype: DType,
    shape: Shape,
    layout: Layout,
    device: Device,
}

impl<'a> TensorViewMut<'a> {
    /// Build a mutable view over raw bytes, validating length and alignment.
    pub fn from_slice(
        data: &'a mut [u8],
        dtype: DType,
        shape: Shape,
        layout: Layout,
    ) -> TensorResult<Self> {
        validate(data.len(), data.as_ptr() as usize, dtype, &shape)?;
        Ok(TensorViewMut {
            data,
            dtype,
            shape,
            layout,
            device: Device::Cpu,
        })
    }

    /// Build a contiguous `F32` mutable view over a typed slice.
    pub fn from_f32(data: &'a mut [f32], shape: Shape) -> TensorResult<Self> {
        let byte_len = std::mem::size_of_val(data);
        let bytes =
            unsafe { std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, byte_len) };
        Self::from_slice(bytes, DType::F32, shape, Layout::Contiguous)
    }

    /// Element dtype.
    pub fn dtype(&self) -> DType {
        self.dtype
    }

    /// Tensor shape.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Memory layout.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Device placement.
    pub fn device(&self) -> Device {
        self.device
    }

    /// Byte length.
    pub fn num_bytes(&self) -> usize {
        self.data.len()
    }

    /// The raw bytes, mutably.
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        self.data
    }

    /// Reborrow as a shorter-lived mutable view over the same memory.
    ///
    /// Lets a `&mut TensorViewMut` be handed to an inference backend as an
    /// output binding without moving the original (which the caller still owns).
    pub fn reborrow(&mut self) -> TensorViewMut<'_> {
        TensorViewMut {
            data: &mut *self.data,
            dtype: self.dtype,
            shape: self.shape.clone(),
            layout: self.layout,
            device: self.device,
        }
    }

    /// A read-only view of the same memory.
    pub fn as_view(&self) -> TensorView<'_> {
        TensorView {
            data: self.data,
            dtype: self.dtype,
            shape: &self.shape,
            layout: self.layout,
            device: self.device,
        }
    }

    /// Mutable `&mut [f32]` access, if the dtype matches.
    pub fn as_f32_mut(&mut self) -> TensorResult<&mut [f32]> {
        if self.dtype != DType::F32 {
            return Err(TensorError::DTypeMismatch {
                actual: self.dtype,
                requested: DType::F32,
            });
        }
        // SAFETY: validated alignment and length multiple at construction.
        Ok(unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut f32,
                self.data.len() / size_of::<f32>(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deliverable_from_slice() {
        // The Milestone 1 deliverable line from the plan.
        let data = [0u8; 48]; // 12 f32 = 48 bytes
        let shape = Shape::from([1, 12]);
        let view = TensorView::from_slice(&data, DType::F32, &shape, Layout::Contiguous).unwrap();
        assert_eq!(view.num_elements(), 12);
        assert!(view.is_contiguous());
        assert_eq!(view.as_f32().unwrap().len(), 12);
    }

    #[test]
    fn from_f32_zero_copy_roundtrip() {
        let data: Vec<f32> = (0..12).map(|i| i as f32).collect();
        let shape = Shape::from([3, 4]);
        let view = TensorView::from_f32(&data, &shape).unwrap();
        let seen = view.as_f32().unwrap();
        assert_eq!(seen, data.as_slice());
        // Zero-copy: the view points at the caller's buffer.
        assert_eq!(seen.as_ptr(), data.as_ptr());
    }

    #[test]
    fn rejects_wrong_byte_length() {
        let data = [0u8; 40]; // not a multiple that fits [1,12] f32 (needs 48)
        let shape = Shape::from([1, 12]);
        let err =
            TensorView::from_slice(&data, DType::F32, &shape, Layout::Contiguous).unwrap_err();
        assert!(matches!(err, TensorError::ByteLength { .. }));
    }

    #[test]
    fn rejects_misaligned_buffer() {
        // Force a 1-byte-offset (odd) pointer into a wider buffer.
        let backing = [0u8; 9];
        let misaligned = &backing[1..9]; // 8 bytes for [1,2] f32... but ptr may be odd
        let shape = Shape::from([1, 2]);
        // Only assert the alignment path when the slice is actually misaligned.
        if !(misaligned.as_ptr() as usize).is_multiple_of(4) {
            let err = TensorView::from_slice(misaligned, DType::F32, &shape, Layout::Contiguous)
                .unwrap_err();
            assert!(matches!(err, TensorError::Alignment { .. }));
        }
    }

    #[test]
    fn dtype_mismatch_on_typed_access() {
        let data = [0u8; 8];
        let shape = Shape::from([8]);
        let view = TensorView::from_slice(&data, DType::U8, &shape, Layout::Contiguous).unwrap();
        assert!(matches!(
            view.as_f32(),
            Err(TensorError::DTypeMismatch { .. })
        ));
        assert_eq!(view.as_u8().unwrap().len(), 8);
    }

    #[test]
    fn mutable_view_writes_through() {
        let mut data = vec![0.0f32; 4];
        {
            let mut view = TensorViewMut::from_f32(&mut data, Shape::from([2, 2])).unwrap();
            for (i, v) in view.as_f32_mut().unwrap().iter_mut().enumerate() {
                *v = i as f32;
            }
        }
        assert_eq!(data, vec![0.0, 1.0, 2.0, 3.0]);
    }
}
