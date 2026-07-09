//! Errors raised while constructing or validating tensors.

use crate::{DType, Shape};

/// Something went wrong building a tensor view or owned tensor.
#[derive(Debug, thiserror::Error)]
pub enum TensorError {
    /// The byte buffer length does not match `shape × element_size`.
    #[error("byte length mismatch for {dtype} {shape}: expected {expected} bytes, got {actual}")]
    ByteLength {
        dtype: DType,
        shape: Shape,
        expected: usize,
        actual: usize,
    },

    /// The buffer's start address is not aligned for the element type.
    #[error("buffer at {ptr:#x} is not {align}-byte aligned for {dtype}")]
    Alignment {
        dtype: DType,
        align: usize,
        ptr: usize,
    },

    /// A typed accessor was called for the wrong dtype.
    #[error("dtype mismatch: tensor holds {actual}, requested {requested}")]
    DTypeMismatch { actual: DType, requested: DType },

    /// Strides were supplied whose length does not match the shape's rank.
    #[error("stride rank mismatch: shape has rank {shape_rank}, got {stride_len} strides")]
    StrideRank {
        shape_rank: usize,
        stride_len: usize,
    },

    /// A sensor adapter or pipeline transform could not build a valid tensor
    /// (e.g. a padded image row that breaks the zero-copy contract).
    #[error("adapter error: {0}")]
    Adapter(String),
}

/// Convenience result alias for tensor construction.
pub type TensorResult<T> = Result<T, TensorError>;

impl From<TensorError> for clankers_core::RobotError {
    fn from(e: TensorError) -> Self {
        clankers_core::RobotError::Other(e.to_string())
    }
}
