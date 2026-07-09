//! # Robotics-focused tensor utilities
//!
//! Zero-copy tensor views for inference, plus high-level image preprocessing helpers.
//!
//! ## Zero-copy inference path
//!
//! Borrow sensor memory directly — no extra clone before handing tensors to
//! [`clankers_ml::Model::run_named`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html#method.run_named):
//!
//! ```no_run
//! use clankers_tensor::{DType, Layout, Shape, TensorView};
//!
//! // Camera frame bytes borrowed from replay or a driver buffer
//! let frame: &[u8] = &[0u8; 480 * 640 * 3];
//! let shape = Shape::from([480, 640, 3]);
//! let view = TensorView::from_slice(
//!     frame,
//!     DType::U8,
//!     &shape,
//!     Layout::Contiguous,
//! )?;
//! # Ok::<(), clankers_tensor::TensorError>(())
//! ```
//!
//! ## Image preprocessing
//!
//! [`ImageTensor`] converts ROS-style [`clankers_ros2::ImageMsg`] into normalized NCHW
//! `f32` tensors, then exposes [`ImageTensor::as_nchw_view`] for zero-copy binding:
//!
//! ```no_run
//! use clankers_ros2::ImageMsg;
//! use clankers_tensor::ImageTensor;
//!
//! let frame = ImageMsg::new(640, 480, vec![128u8; 640 * 480 * 3]);
//! let tensor = ImageTensor::from_ros_msg(&frame)?
//!     .resize(224, 224)?
//!     .normalize_imagenet()?
//!     .to_nchw()?;
//! let shape = tensor.nchw_shape();
//! let view = tensor.as_nchw_view(&shape)?;
//! # Ok::<(), clankers_core::RobotError>(())
//! ```
//!
//! ## Core primitives
//!
//! | Type | Role |
//! |------|------|
//! | [`TensorView`] / [`TensorViewMut`] | Borrowed slices for inference I/O |
//! | [`Tensor`] | Owned buffer returned from inference |
//! | [`Shape`] / [`ShapeSpec`] | Static or dynamic-rank shapes |
//! | [`DType`] / [`Layout`] | Element type and memory order |
//! | [`TensorArena`] | Scratch allocation for hot loops |

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
