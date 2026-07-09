//! Composable preprocessing transforms over owned [`Tensor`](crate::Tensor)s.
//!
//! A [`Transform`] maps one tensor to another; [`Transform::then`] chains them so
//! a preprocessing pipeline reads as data flow:
//!
//! ```
//! use clankers_tensor::{DType, Shape, Tensor};
//! use clankers_tensor::pipeline::{Normalize, ToF32, Transform};
//!
//! // A 1x1 RGB pixel as raw u8, normalised the way a vision model expects.
//! let pixel = Tensor::from_bytes(DType::U8, Shape::from([1, 1, 3]), &[255, 128, 0]).unwrap();
//! let pipeline = ToF32::pixels().then(Normalize::imagenet());
//! let out = pipeline.apply(pixel).unwrap();
//! assert_eq!(out.dtype(), DType::F32);
//! ```

pub mod transforms;

pub use transforms::{Chain, Normalize, ToF32, Transform};
