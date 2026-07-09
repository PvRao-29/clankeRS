//! The bridge tensor passed across the backend boundary.

use clankers_tensor::{DType, Shape, Tensor, TensorView, TensorViewMut};

/// A tensor handed to (or produced by) a [`BackendSession`].
///
/// It is one of:
/// * [`Borrowed`](BackendTensor::Borrowed) — a zero-copy read-only view over
///   caller/arena memory; the norm for inputs.
/// * [`Owned`](BackendTensor::Owned) — an owned tensor; the norm for outputs the
///   backend produces for a dynamic shape.
/// * [`BorrowedMut`](BackendTensor::BorrowedMut) — a mutable view over a
///   caller-preallocated output buffer, filled in place by `run_into` on backends
///   that advertise [`supports_preallocated_outputs`]. No allocation occurs.
///
/// [`BackendSession`]: crate::backend::BackendSession
/// [`supports_preallocated_outputs`]: crate::backend::BackendCapabilities::supports_preallocated_outputs
pub enum BackendTensor<'a> {
    /// A borrowed, read-only view — no allocation, no copy.
    Borrowed(TensorView<'a>),
    /// An owned tensor.
    Owned(Tensor),
    /// A borrowed, writable output buffer the backend fills in place.
    BorrowedMut(TensorViewMut<'a>),
}

impl<'a> BackendTensor<'a> {
    /// Borrow this tensor as a read-only view.
    pub fn view(&self) -> TensorView<'_> {
        match self {
            BackendTensor::Borrowed(v) => *v,
            BackendTensor::Owned(t) => t.view(),
            BackendTensor::BorrowedMut(v) => v.as_view(),
        }
    }

    /// The element dtype.
    pub fn dtype(&self) -> DType {
        self.view().dtype()
    }

    /// The shape (cloned out of the underlying view).
    pub fn shape(&self) -> Shape {
        self.view().shape().clone()
    }

    /// Byte length.
    pub fn num_bytes(&self) -> usize {
        self.view().num_bytes()
    }

    /// If this is a preallocated output binding, the writable byte buffer to fill
    /// in place; `None` for borrowed/owned tensors.
    pub fn bytes_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            BackendTensor::BorrowedMut(v) => Some(v.bytes_mut()),
            _ => None,
        }
    }

    /// Materialise an owned, contiguous copy of this tensor.
    ///
    /// For an already-`Owned` tensor this still copies; use [`BackendTensor::into_owned`]
    /// to move without copying when you have ownership.
    pub fn to_owned_tensor(&self) -> Tensor {
        let v = self.view();
        Tensor::from_bytes(v.dtype(), v.shape().clone(), v.bytes())
            .expect("a valid view always yields a valid owned tensor")
    }

    /// Convert into an owned tensor, moving when already owned and copying when
    /// borrowed.
    pub fn into_owned(self) -> Tensor {
        match self {
            BackendTensor::Owned(t) => t,
            BackendTensor::Borrowed(v) => {
                Tensor::from_bytes(v.dtype(), v.shape().clone(), v.bytes())
                    .expect("a valid view always yields a valid owned tensor")
            }
            BackendTensor::BorrowedMut(v) => {
                let view = v.as_view();
                Tensor::from_bytes(view.dtype(), view.shape().clone(), view.bytes())
                    .expect("a valid view always yields a valid owned tensor")
            }
        }
    }
}

impl<'a> From<TensorView<'a>> for BackendTensor<'a> {
    fn from(v: TensorView<'a>) -> Self {
        BackendTensor::Borrowed(v)
    }
}

impl From<Tensor> for BackendTensor<'static> {
    fn from(t: Tensor) -> Self {
        BackendTensor::Owned(t)
    }
}
