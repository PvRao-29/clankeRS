//! Borrow a flat `f32` state / observation vector as a tensor.
//!
//! Robot policies typically take a proprioceptive state vector (joint angles,
//! velocities, goal deltas). [`StateInput`] borrows such a slice and lends a
//! contiguous `F32` [`TensorView`] shaped for the model.

use crate::error::{TensorError, TensorResult};
use crate::{Shape, TensorView};

/// A zero-copy borrow of an `f32` slice as an `F32` tensor of a chosen shape.
pub struct StateInput<'a> {
    data: &'a [f32],
    shape: Shape,
}

impl<'a> StateInput<'a> {
    /// Borrow `data` with an explicit `shape`, validating the element count.
    pub fn new(data: &'a [f32], shape: Shape) -> TensorResult<Self> {
        if data.len() != shape.num_elements() {
            return Err(TensorError::Adapter(format!(
                "state length {} does not match shape {shape} ({} elements)",
                data.len(),
                shape.num_elements()
            )));
        }
        Ok(StateInput { data, shape })
    }

    /// Borrow `data` as a batched row vector of shape `[1, len]` — the common
    /// single-observation case.
    pub fn vector(data: &'a [f32]) -> Self {
        let shape = Shape::from([1, data.len()]);
        StateInput { data, shape }
    }

    /// The tensor shape.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Borrow the state as a read-only `F32` [`TensorView`].
    pub fn view(&self) -> TensorView<'_> {
        TensorView::from_f32(self.data, &self.shape)
            .expect("state length validated at construction")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DType;

    #[test]
    fn vector_is_row_shaped_and_zero_copy() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let state = StateInput::vector(&data);
        assert_eq!(state.shape(), &Shape::from([1, 4]));
        let view = state.view();
        assert_eq!(view.dtype(), DType::F32);
        assert_eq!(view.as_f32().unwrap().as_ptr(), data.as_ptr());
    }

    #[test]
    fn explicit_shape_validated() {
        let data = vec![0.0f32; 12];
        assert!(StateInput::new(&data, Shape::from([3, 4])).is_ok());
        assert!(matches!(
            StateInput::new(&data, Shape::from([3, 3])),
            Err(TensorError::Adapter(_))
        ));
    }
}
