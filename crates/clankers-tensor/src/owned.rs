//! Owned tensors — heap-backed storage the engine can allocate, reuse, and hand
//! back as inference outputs.

use crate::buffer::Buffer;
use crate::error::{TensorError, TensorResult};
use crate::view::{TensorView, TensorViewMut};
use crate::{DType, Device, Layout, Shape};

/// An owned, contiguous tensor with an 8-byte-aligned backing [`Buffer`].
///
/// `Tensor` is what the engine returns from `run` and what the arena hands out
/// for conversions and preallocated outputs. It is always row-major contiguous;
/// strided data is represented by borrowing through a [`TensorView`] instead.
#[derive(Debug, Clone)]
pub struct Tensor {
    buffer: Buffer,
    dtype: DType,
    shape: Shape,
    device: Device,
}

impl Tensor {
    /// A zero-initialised tensor of the given dtype and shape.
    pub fn zeros(dtype: DType, shape: Shape) -> Self {
        let buffer = Buffer::zeroed(shape.num_bytes(dtype.element_size()));
        Tensor {
            buffer,
            dtype,
            shape,
            device: Device::Cpu,
        }
    }

    /// Build an `F32` tensor from a vector, checking the length against `shape`.
    pub fn from_f32_vec(shape: Shape, data: Vec<f32>) -> TensorResult<Self> {
        if data.len() != shape.num_elements() {
            return Err(TensorError::ByteLength {
                dtype: DType::F32,
                shape: shape.clone(),
                expected: shape.num_bytes(DType::F32.element_size()),
                actual: data.len() * DType::F32.element_size(),
            });
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(&data[..])) };
        Ok(Tensor {
            buffer: Buffer::from_bytes(bytes),
            dtype: DType::F32,
            shape,
            device: Device::Cpu,
        })
    }

    /// Build a tensor by copying raw bytes (length validated against the shape).
    pub fn from_bytes(dtype: DType, shape: Shape, bytes: &[u8]) -> TensorResult<Self> {
        let expected = shape.num_bytes(dtype.element_size());
        if bytes.len() != expected {
            return Err(TensorError::ByteLength {
                dtype,
                shape,
                expected,
                actual: bytes.len(),
            });
        }
        Ok(Tensor {
            buffer: Buffer::from_bytes(bytes),
            dtype,
            shape,
            device: Device::Cpu,
        })
    }

    /// Element dtype.
    pub fn dtype(&self) -> DType {
        self.dtype
    }

    /// Tensor shape.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Device placement.
    pub fn device(&self) -> Device {
        self.device
    }

    /// Number of elements.
    pub fn num_elements(&self) -> usize {
        self.shape.num_elements()
    }

    /// Byte length.
    pub fn num_bytes(&self) -> usize {
        self.buffer.len()
    }

    /// The raw bytes.
    pub fn bytes(&self) -> &[u8] {
        self.buffer.as_bytes()
    }

    /// The raw bytes, mutably.
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut_bytes()
    }

    /// Borrow the whole tensor as a read-only [`TensorView`] (always contiguous).
    pub fn view(&self) -> TensorView<'_> {
        // Length and alignment are guaranteed by construction, so this cannot fail.
        TensorView::from_slice(self.buffer.as_bytes(), self.dtype, &self.shape, Layout::Contiguous)
            .expect("owned tensor is always a valid contiguous view")
    }

    /// Borrow the whole tensor as a writable [`TensorViewMut`].
    pub fn view_mut(&mut self) -> TensorViewMut<'_> {
        let dtype = self.dtype;
        let shape = self.shape.clone();
        TensorViewMut::from_slice(self.buffer.as_mut_bytes(), dtype, shape, Layout::Contiguous)
            .expect("owned tensor is always a valid contiguous view")
    }

    /// Read the data as `&[f32]`, if the dtype matches.
    pub fn as_f32(&self) -> TensorResult<&[f32]> {
        if self.dtype != DType::F32 {
            return Err(TensorError::DTypeMismatch {
                actual: self.dtype,
                requested: DType::F32,
            });
        }
        // SAFETY: buffer is 8-byte aligned (≥ f32 align) and holds a whole number
        // of f32 by construction.
        Ok(unsafe {
            std::slice::from_raw_parts(
                self.buffer.as_bytes().as_ptr() as *const f32,
                self.num_elements(),
            )
        })
    }

    /// Copy the data out as an owned `Vec<f32>`.
    pub fn to_f32_vec(&self) -> TensorResult<Vec<f32>> {
        Ok(self.as_f32()?.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeros_has_right_size() {
        let t = Tensor::zeros(DType::F32, Shape::from([1, 3, 4, 4]));
        assert_eq!(t.num_elements(), 48);
        assert_eq!(t.num_bytes(), 48 * 4);
        assert!(t.as_f32().unwrap().iter().all(|&x| x == 0.0));
    }

    #[test]
    fn from_f32_vec_roundtrips_via_view() {
        let data: Vec<f32> = (0..12).map(|i| i as f32).collect();
        let t = Tensor::from_f32_vec(Shape::from([3, 4]), data.clone()).unwrap();
        assert_eq!(t.to_f32_vec().unwrap(), data);
        // The borrowed view sees the same values.
        assert_eq!(t.view().as_f32().unwrap(), data.as_slice());
    }

    #[test]
    fn from_f32_vec_rejects_wrong_length() {
        let err = Tensor::from_f32_vec(Shape::from([3, 4]), vec![0.0; 5]).unwrap_err();
        assert!(matches!(err, TensorError::ByteLength { .. }));
    }

    #[test]
    fn mutable_view_edits_owned_storage() {
        let mut t = Tensor::zeros(DType::F32, Shape::from([2, 2]));
        {
            let mut vm = t.view_mut();
            vm.as_f32_mut().unwrap()[3] = 9.0;
        }
        assert_eq!(t.as_f32().unwrap()[3], 9.0);
    }
}
