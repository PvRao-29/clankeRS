//! Robotics-focused tensor utilities for clankeRS.
//!
//! The crate is built around a small set of zero-copy primitives:
//!
//! * [`DType`], [`Shape`]/[`ShapeSpec`], [`Layout`], and [`Device`] describe a
//!   tensor's element type, extent, memory order, and placement.
//! * [`TensorView`] / [`TensorViewMut`] borrow memory the caller already owns
//!   (a decoded frame, a state vector, an arena slot) without copying.
//! * [`Tensor`] owns an 8-byte-aligned [`buffer::Buffer`] and is what inference
//!   returns.
//!
//! [`ImageTensor`] is the existing higher-level image preprocessing helper and
//! is retained during the migration to the view/owned primitives.

pub mod adapters;
pub mod arena;
pub mod buffer;
pub mod device;
pub mod dtype;
pub mod error;
pub mod image;
pub mod layout;
pub mod owned;
pub mod pipeline;
pub mod pointcloud;
pub mod shape;
pub mod view;

pub use adapters::{ImageInput, StateInput};
pub use arena::{AllocationPolicy, TensorArena};
pub use device::Device;
pub use dtype::DType;
pub use error::{TensorError, TensorResult};
pub use image::ImageTensor;
pub use layout::{DataLayout, Layout};
pub use owned::Tensor;
pub use pointcloud::PointCloudTensor;
pub use shape::{Dim, Shape, ShapeSpec};
pub use view::{TensorView, TensorViewMut};
