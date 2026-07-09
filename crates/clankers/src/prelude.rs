//! Common imports for clankeRS robot nodes.
//!
//! Pull this in at the top of every node, example, and replay test:
//!
//! ```
//! use clankers::prelude::*;
//! ```
//!
//! ## What you get
//!
//! * **Runtime** — [`RobotContext`], [`RobotNode`], [`RobotResult`]
//! * **Pub/sub** — [`Publisher`], [`Subscriber`], [`ImageMsg`], [`DetectionArray`], [`QosProfile`]
//! * **Inference** — [`Model`], [`ModelBuilder`], [`TensorView`], [`NamedOutputs`]
//! * **Sensors** — [`ImageTensor`], [`ImageInput`], [`StateInput`]
//! * **Data** — [`McapLog`], [`Replay`]
//! * **Testing** — [`ReplayContext`], [`replay_test`], assertion helpers
//! * **Macros** — [`node`], [`replay_test`]

pub use crate::{
    assert_dropped_messages, assert_max_latency, assert_no_panics, assert_topic_exists, node,
    replay_test, ClankeRSConfig, Detection, DetectionArray, ImageInput, ImageMsg, ImageTensor,
    InferenceEngine, InferenceStats, LatencyStats, McapLog, Model, ModelBuilder, ModelEngine,
    NamedOutputs, Publisher, QosProfile, Replay, ReplayContext, ReplayTestResult, RobotContext,
    RobotError, RobotNode, RobotResult, RobotRuntime, Shape, StateInput, Subscriber, Tensor,
    TensorView, Timestamp, TopicName,
};

pub use std::time::Duration;
