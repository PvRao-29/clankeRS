//! # clankeRS — Rust SDK for robotics
//!
//! **Train in PyTorch. Deploy in Rust. Replay-test against real robot logs.**
//!
//! This crate is the umbrella facade. Most applications depend only on
//! this package and import [`prelude`] for everyday node code.
//!
//! ## Quick start — a robot node
//!
//! ```no_run
//! use clankers::prelude::*;
//!
//! #[clankers::node]
//! async fn main(ctx: RobotContext) -> RobotResult<()> {
//!     let _node = RobotNode::new(ctx.node_name().as_str()).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Quick start — optimized inference
//!
//! [`Model`] is the main inference API. Bind zero-copy [`TensorView`] inputs and
//! read named outputs. See [`ml`] for the full surface.
//!
//! ```no_run
//! # #[cfg(feature = "ml")]
//! # fn example() -> clankers::RobotResult<()> {
//! use clankers::ml::OnnxRuntimeBackend;
//! use clankers::prelude::*;
//! use clankers_tensor::{DType, Layout, Shape, TensorView};
//!
//! let mut model = Model::builder()
//!     .backend(OnnxRuntimeBackend::default())
//!     .load("models/policy.onnx")?;
//!
//! let image_shape = Shape::from([1, 64, 64, 3]);
//! let image = TensorView::from_slice(
//!     &[0u8; 64 * 64 * 3],
//!     DType::U8,
//!     &image_shape,
//!     Layout::Contiguous,
//! )?;
//! let state_shape = Shape::from([1, 12]);
//! let state = TensorView::from_f32(&[0.0f32; 12], &state_shape)?;
//!
//! let outputs = model.run_named([("image", image), ("state", state)])?;
//! let _action = outputs.get("action");
//! # Ok(())
//! # }
//! ```
//!
//! ## Module guide
//!
//! | Module | When to use it |
//! |--------|----------------|
//! | [`prelude`] | One import for nodes, inference, pub/sub, and replay tests |
//! | [`ros2`] | Sim pub/sub — [`RobotNode`], [`ImageMsg`], [`DetectionArray`] |
//! | [`ml`] / [`Model`] | Load ONNX models, run inference (start here) |
//! | [`tensor`] | [`TensorView`], [`ImageTensor`] preprocessing |
//! | [`data`] | MCAP inspect, replay, compare |
//! | [`testing`] | [`ReplayContext`] and replay assertions |
//! | [`inference`] | Power-user [`InferenceEngine`] and backends |
//! | [`runtime`] | [`RobotRuntime`] metrics and tracing helpers |
//!
//! ## Workspace crates
//!
//! The facade re-exports these focused crates (each has its own [docs.rs](https://docs.rs/clankers) page):
//! `clankers-core`, `clankers-ros2`, `clankers-tensor`, `clankers-ml`, `clankers-data`,
//! `clankers-testing`, `clankers-geometry`, `clankers-runtime`, `clankers-macros`.
//!
//! Install the CLI separately: `cargo install clankers-cli`.

pub mod prelude;

pub use clankers_core::{
    ClankeRSConfig, LatencyStats, ModelBackendKind, RobotContext, RobotError, RobotResult,
    Timestamp, TopicName,
};
pub use clankers_data::{InspectReport, McapLog, Replay, ReplayResult};
pub use clankers_geometry::{Pose, Transform, Twist};
pub use clankers_macros::{node, replay_test};
#[cfg(feature = "ml")]
pub use clankers_ml::onnx_engine_from_config;
pub use clankers_ml::{
    engine_from_model_config, noop_engine_from_config, ConfiguredEngine, InferenceEngine,
    InferenceError, InferenceStats, Model, ModelBuilder, ModelEngine, ModelValidator, NamedOutputs,
    RuntimeBackend, ValidationReport,
};
pub use clankers_ros2::{
    Detection, DetectionArray, ImageMsg, Publisher, QosProfile, RobotNode, Subscriber,
};
// `inject_message` feeds the in-memory sim bus (used by replay). The `clankers`
// crate always uses the sim backend; the real rclrs/DDS backend is a separate
// colcon package (ros2/clankers-ros2-dds) where messages arrive over DDS.
pub use clankers_ros2::inject_message;
pub use clankers_runtime::{RobotRuntime, RuntimeMetrics};
pub use clankers_tensor::{ImageInput, ImageTensor, Shape, StateInput, Tensor, TensorView};
pub use clankers_testing::{
    assert_dropped_messages, assert_max_latency, assert_no_panics, assert_topic_exists,
    AggregatedInferenceStats, ReplayContext, ReplayTestResult,
};

pub mod runtime {
    pub use clankers_runtime::runtime::*;
}

pub mod testing {
    pub use clankers_testing::*;
}

pub mod data {
    pub use clankers_data::*;
}

pub mod ml {
    pub use clankers_ml::*;
}

/// Lower-level inference runtime used by [`Model`].
///
/// Most applications should use [`Model`]. Construct an [`InferenceEngine`] directly when
/// implementing custom backends, allocation policies, or advanced integrations.
pub mod inference {
    pub use clankers_ml::inference::*;
}

/// Inference backends and the tensor specs / capabilities they report.
pub mod backend {
    #[allow(deprecated)]
    pub use clankers_ml::backend::*;
}

pub mod tensor {
    pub use clankers_tensor::*;
}

pub mod ros2 {
    pub use clankers_ros2::*;
}
